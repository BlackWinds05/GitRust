use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::helpers::pagination::Pagination;
use crate::issues::model::{
    AssigneeInfo, CommentWithAuthor, Issue, IssueComment, IssueDetail, IssueListItem, LabelInfo,
};
use crate::markdown::render::render_markdown;

pub async fn create_issue(
    pool: &PgPool,
    repo_id: Uuid,
    author_id: Uuid,
    title: &str,
    description: &str,
    milestone_id: Option<Uuid>,
    label_ids: &[Uuid],
    assignee_ids: &[Uuid],
    due_date: Option<NaiveDate>,
) -> AppResult<Issue> {
    let number: (i32,) = sqlx::query_as(
        "SELECT COALESCE(MAX(number), 0) + 1 FROM issues WHERE repository_id = $1",
    )
    .bind(repo_id)
    .fetch_one(pool)
    .await?;

    let issue = sqlx::query_as::<_, Issue>(
        r#"INSERT INTO issues (repository_id, number, title, description, author_id, milestone_id, due_date)
           VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"#,
    )
    .bind(repo_id)
    .bind(number.0)
    .bind(title)
    .bind(description)
    .bind(author_id)
    .bind(milestone_id)
    .bind(due_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)?;

    for lid in label_ids {
        sqlx::query(
            "INSERT INTO issue_label_assignments (issue_id, label_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(issue.id)
        .bind(lid)
        .execute(pool)
        .await?;
    }

    for uid in assignee_ids {
        sqlx::query(
            "INSERT INTO issue_assignees (issue_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(issue.id)
        .bind(uid)
        .execute(pool)
        .await?;
    }

    Ok(issue)
}

pub async fn list_issues(
    pool: &PgPool,
    repo_id: Uuid,
    state: Option<&str>,
    page: u32,
    per_page: u32,
    milestone: Option<&str>,
    label: Option<&str>,
    assignee: Option<&str>,
    author: Option<&str>,
    search: Option<&str>,
) -> AppResult<(Vec<IssueListItem>, Pagination)> {
    let state_filter = state.unwrap_or("open");

    let mut count_sql = String::from(
        "SELECT COUNT(*) FROM issues i WHERE i.repository_id = $1 AND i.state = $2",
    );
    let mut list_sql = String::from(
        r#"SELECT i.id, i.number, i.title, i.state,
                  u.username as author_username, u.display_name as author_display_name,
                  i.created_at,
                  (SELECT COUNT(*) FROM issue_label_assignments WHERE issue_id = i.id) as label_count
           FROM issues i
           JOIN users u ON i.author_id = u.id
           WHERE i.repository_id = $1 AND i.state = $2"#,
    );

    let mut extra_params: Vec<String> = vec![];
    let mut param_idx = 3i32;

    if let Some(m) = milestone {
        extra_params.push(m.to_string());
        param_idx += 1;
        count_sql.push_str(&format!(
            " AND i.milestone_id = (SELECT id FROM milestones WHERE repository_id = $1 AND title = ${})",
            param_idx - 1
        ));
        list_sql.push_str(&format!(
            " AND i.milestone_id = (SELECT id FROM milestones WHERE repository_id = $1 AND title = ${})",
            param_idx - 1
        ));
    }

    if let Some(l) = label {
        extra_params.push(l.to_string());
        param_idx += 1;
        count_sql.push_str(&format!(
            " AND EXISTS (SELECT 1 FROM issue_label_assignments ila JOIN issue_labels il ON ila.label_id = il.id WHERE ila.issue_id = i.id AND il.name = ${})",
            param_idx - 1
        ));
        list_sql.push_str(&format!(
            " AND EXISTS (SELECT 1 FROM issue_label_assignments ila JOIN issue_labels il ON ila.label_id = il.id WHERE ila.issue_id = i.id AND il.name = ${})",
            param_idx - 1
        ));
    }

    if let Some(a) = assignee {
        extra_params.push(a.to_string());
        param_idx += 1;
        count_sql.push_str(&format!(
            " AND EXISTS (SELECT 1 FROM issue_assignees ia JOIN users u2 ON ia.user_id = u2.id WHERE ia.issue_id = i.id AND u2.username = ${})",
            param_idx - 1
        ));
        list_sql.push_str(&format!(
            " AND EXISTS (SELECT 1 FROM issue_assignees ia JOIN users u2 ON ia.user_id = u2.id WHERE ia.issue_id = i.id AND u2.username = ${})",
            param_idx - 1
        ));
    }

    if let Some(auth) = author {
        extra_params.push(auth.to_string());
        param_idx += 1;
        count_sql.push_str(&format!(" AND u.username = ${}", param_idx - 1));
        list_sql.push_str(&format!(" AND u.username = ${}", param_idx - 1));
    }

    if let Some(q) = search {
        extra_params.push(format!("%{}%", q));
        param_idx += 1;
        count_sql.push_str(&format!(" AND i.title ILIKE ${}", param_idx - 1));
        list_sql.push_str(&format!(" AND i.title ILIKE ${}", param_idx - 1));
    }

    list_sql.push_str(" ORDER BY i.created_at DESC");

    let total: (i64,) = {
        let mut q = sqlx::query_as(&count_sql).bind(repo_id).bind(state_filter);
        for p in &extra_params {
            q = q.bind(p);
        }
        q.fetch_one(pool).await?
    };
    let pagination = Pagination::new(page, per_page, total.0 as u64);

    let limit_p = param_idx;
    let offset_p = param_idx + 1;
    list_sql.push_str(&format!(" LIMIT ${} OFFSET ${}", limit_p, offset_p));

    let mut q = sqlx::query_as::<_, IssueListItem>(&list_sql)
        .bind(repo_id)
        .bind(state_filter);
    for p in &extra_params {
        q = q.bind(p);
    }
    let issues = q
        .bind(per_page as i64)
        .bind(pagination.offset() as i64)
        .fetch_all(pool)
        .await?;

    Ok((issues, pagination))
}

pub async fn get_issue(pool: &PgPool, repo_id: Uuid, number: i32) -> AppResult<IssueDetail> {
    let issue = sqlx::query_as::<_, Issue>(
        "SELECT * FROM issues WHERE repository_id = $1 AND number = $2",
    )
    .bind(repo_id)
    .bind(number)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Issue not found.".into()))?;

    #[derive(sqlx::FromRow)]
    struct AuthorRow {
        username: String,
        display_name: String,
    }
    let author = sqlx::query_as::<_, AuthorRow>(
        "SELECT username, display_name FROM users WHERE id = $1",
    )
    .bind(issue.author_id)
    .fetch_one(pool)
    .await?;

    let labels = sqlx::query_as::<_, LabelInfo>(
        r#"SELECT il.id, il.name, il.color
           FROM issue_labels il
           JOIN issue_label_assignments ila ON il.id = ila.label_id
           WHERE ila.issue_id = $1"#,
    )
    .bind(issue.id)
    .fetch_all(pool)
    .await?;

    let milestone_title = if let Some(mid) = issue.milestone_id {
        sqlx::query_scalar::<_, String>("SELECT title FROM milestones WHERE id = $1")
            .bind(mid)
            .fetch_optional(pool)
            .await?
    } else {
        None
    };

    let assignees = sqlx::query_as::<_, (Uuid, String, String)>(
        r#"SELECT u.id, u.username, u.display_name
           FROM users u
           JOIN issue_assignees ia ON u.id = ia.user_id
           WHERE ia.issue_id = $1"#,
    )
    .bind(issue.id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(id, username, display_name)| AssigneeInfo {
        id,
        username,
        display_name,
    })
    .collect();

    let description_html = issue
        .description
        .as_ref()
        .map(|d| render_markdown(d));

    Ok(IssueDetail {
        id: issue.id,
        number: issue.number,
        title: issue.title,
        description: issue.description,
        description_html,
        state: issue.state,
        author_id: issue.author_id,
        author_username: author.username,
        author_display_name: author.display_name,
        due_date: issue.due_date,
        created_at: issue.created_at,
        updated_at: issue.updated_at,
        closed_at: issue.closed_at,
        labels,
        milestone_title,
        assignees,
    })
}

pub async fn update_issue(
    pool: &PgPool,
    repo_id: Uuid,
    number: i32,
    title: &str,
    description: &str,
    milestone_id: Option<Uuid>,
    assignee_ids: &[Uuid],
    label_ids: &[Uuid],
    due_date: Option<NaiveDate>,
) -> AppResult<Issue> {
    let issue = sqlx::query_as::<_, Issue>(
        "SELECT * FROM issues WHERE repository_id = $1 AND number = $2",
    )
    .bind(repo_id)
    .bind(number)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Issue not found.".into()))?;

    sqlx::query(
        "UPDATE issues SET title=$1, description=$2, milestone_id=$3, due_date=$4, updated_at=now() WHERE id=$5",
    )
    .bind(title)
    .bind(description)
    .bind(milestone_id)
    .bind(due_date)
    .bind(issue.id)
    .execute(pool)
    .await?;

    sqlx::query("DELETE FROM issue_label_assignments WHERE issue_id=$1")
        .bind(issue.id)
        .execute(pool)
        .await?;
    for lid in label_ids {
        sqlx::query(
            "INSERT INTO issue_label_assignments (issue_id, label_id) VALUES ($1,$2) ON CONFLICT DO NOTHING",
        )
        .bind(issue.id)
        .bind(lid)
        .execute(pool)
        .await?;
    }

    sqlx::query("DELETE FROM issue_assignees WHERE issue_id=$1")
        .bind(issue.id)
        .execute(pool)
        .await?;
    for uid in assignee_ids {
        sqlx::query(
            "INSERT INTO issue_assignees (issue_id, user_id) VALUES ($1,$2) ON CONFLICT DO NOTHING",
        )
        .bind(issue.id)
        .bind(uid)
        .execute(pool)
        .await?;
    }

    sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id=$1")
        .bind(issue.id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
}

pub async fn delete_issue(pool: &PgPool, repo_id: Uuid, number: i32) -> AppResult<()> {
    let r = sqlx::query("DELETE FROM issues WHERE repository_id=$1 AND number=$2")
        .bind(repo_id)
        .bind(number)
        .execute(pool)
        .await?;
    if r.rows_affected() == 0 {
        return Err(AppError::NotFound("Issue not found.".into()));
    }
    Ok(())
}

pub async fn close_issue(pool: &PgPool, repo_id: Uuid, number: i32) -> AppResult<()> {
    let r = sqlx::query(
        "UPDATE issues SET state = 'closed', closed_at = now(), updated_at = now()
         WHERE repository_id = $1 AND number = $2 AND state = 'open'",
    )
    .bind(repo_id)
    .bind(number)
    .execute(pool)
    .await?;
    if r.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Issue not found or already closed.".into(),
        ));
    }
    Ok(())
}

pub async fn reopen_issue(pool: &PgPool, repo_id: Uuid, number: i32) -> AppResult<()> {
    sqlx::query(
        "UPDATE issues SET state = 'open', closed_at = NULL, updated_at = now()
         WHERE repository_id = $1 AND number = $2",
    )
    .bind(repo_id)
    .bind(number)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn assign_label(pool: &PgPool, issue_id: Uuid, label_id: Uuid) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO issue_label_assignments (issue_id, label_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(issue_id)
    .bind(label_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Comments ──

pub async fn create_comment(
    pool: &PgPool,
    issue_id: Uuid,
    author_id: Uuid,
    body: &str,
) -> AppResult<IssueComment> {
    sqlx::query_as::<_, IssueComment>(
        "INSERT INTO issue_comments (issue_id, author_id, body) VALUES ($1,$2,$3) RETURNING *",
    )
    .bind(issue_id)
    .bind(author_id)
    .bind(body)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn list_comments(pool: &PgPool, issue_id: Uuid) -> AppResult<Vec<CommentWithAuthor>> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: Uuid,
        body: String,
        author_username: String,
        author_display_name: String,
        author_id: Uuid,
        created_at: DateTime<Utc>,
    }

    let rows = sqlx::query_as::<_, Row>(
        r#"SELECT ic.id, ic.body, u.username as author_username, u.display_name as author_display_name, ic.author_id, ic.created_at
           FROM issue_comments ic JOIN users u ON ic.author_id = u.id
           WHERE ic.issue_id = $1 ORDER BY ic.created_at ASC"#,
    )
    .bind(issue_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CommentWithAuthor {
            id: r.id,
            body_html: render_markdown(&r.body),
            body: r.body,
            author_username: r.author_username,
            author_display_name: r.author_display_name,
            author_id: r.author_id,
            created_at: r.created_at,
        })
        .collect())
}

pub async fn delete_comment(
    pool: &PgPool,
    comment_id: Uuid,
    issue_id: Uuid,
) -> AppResult<()> {
    let r = sqlx::query("DELETE FROM issue_comments WHERE id=$1 AND issue_id=$2")
        .bind(comment_id)
        .bind(issue_id)
        .execute(pool)
        .await?;
    if r.rows_affected() == 0 {
        return Err(AppError::NotFound("Comment not found.".into()));
    }
    Ok(())
}
