use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;
use crate::error::{AppError, AppResult};
use crate::milestones::model::{Milestone, MilestoneWithProgress};

pub async fn list(pool: &PgPool, repo_id: Uuid) -> AppResult<Vec<MilestoneWithProgress>> {
    Ok(sqlx::query_as::<_, MilestoneWithProgress>(
        r#"SELECT m.*,
           (SELECT COUNT(*) FROM issues WHERE milestone_id = m.id AND state = 'open') as open_issues,
           (SELECT COUNT(*) FROM issues WHERE milestone_id = m.id AND state = 'closed') as closed_issues
           FROM milestones m WHERE m.repository_id = $1 ORDER BY m.created_at DESC"#,
    ).bind(repo_id).fetch_all(pool).await?)
}

pub async fn create(pool: &PgPool, repo_id: Uuid, title: &str, description: Option<&str>, due_date: Option<NaiveDate>) -> AppResult<Milestone> {
    Ok(sqlx::query_as::<_, Milestone>(
        "INSERT INTO milestones (repository_id, title, description, due_date) VALUES ($1, $2, $3, $4) RETURNING *"
    ).bind(repo_id).bind(title).bind(description).bind(due_date).fetch_one(pool).await?)
}

pub async fn get(pool: &PgPool, repo_id: Uuid, milestone_id: Uuid) -> AppResult<Milestone> {
    sqlx::query_as::<_, Milestone>(
        "SELECT * FROM milestones WHERE id = $1 AND repository_id = $2"
    ).bind(milestone_id).bind(repo_id)
    .fetch_optional(pool).await?
    .ok_or_else(|| AppError::NotFound("Milestone not found.".into()))
}

pub async fn update_milestone(
    pool: &PgPool, id: Uuid, repo_id: Uuid,
    title: &str, description: Option<&str>, due_date: Option<NaiveDate>,
) -> AppResult<Milestone> {
    sqlx::query_as::<_, Milestone>(
        "UPDATE milestones SET title=$1, description=$2, due_date=$3 WHERE id=$4 AND repository_id=$5 RETURNING *"
    ).bind(title).bind(description).bind(due_date).bind(id).bind(repo_id)
    .fetch_optional(pool).await?
    .ok_or_else(|| AppError::NotFound("Milestone not found.".into()))
}

pub async fn toggle_milestone(pool: &PgPool, id: Uuid, repo_id: Uuid) -> AppResult<Milestone> {
    sqlx::query_as::<_, Milestone>(
        "UPDATE milestones SET state = CASE WHEN state='open' THEN 'closed' ELSE 'open' END WHERE id=$1 AND repository_id=$2 RETURNING *"
    ).bind(id).bind(repo_id)
    .fetch_optional(pool).await?
    .ok_or_else(|| AppError::NotFound("Milestone not found.".into()))
}

pub async fn delete_milestone(pool: &PgPool, id: Uuid, repo_id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE issues SET milestone_id = NULL WHERE milestone_id = $1")
        .bind(id).execute(pool).await?;
    let r = sqlx::query("DELETE FROM milestones WHERE id=$1 AND repository_id=$2")
        .bind(id).bind(repo_id).execute(pool).await?;
    if r.rows_affected() == 0 {
        return Err(AppError::NotFound("Milestone not found.".into()));
    }
    Ok(())
}

pub async fn get_issues_for_milestone(pool: &PgPool, milestone_id: Uuid) -> AppResult<Vec<crate::issues::model::IssueListItem>> {
    Ok(sqlx::query_as::<_, crate::issues::model::IssueListItem>(
        r#"SELECT i.id, i.number, i.title, i.state,
                  u.username as author_username, u.display_name as author_display_name,
                  i.created_at,
                  (SELECT COUNT(*) FROM issue_label_assignments WHERE issue_id = i.id) as label_count
           FROM issues i JOIN users u ON i.author_id = u.id
           WHERE i.milestone_id = $1 ORDER BY i.created_at DESC"#,
    ).bind(milestone_id).fetch_all(pool).await?)
}
