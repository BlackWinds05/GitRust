use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}

impl Pagination {
    pub fn new(page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;
        Pagination {
            page: page.max(1),
            per_page,
            total,
            total_pages: total_pages.max(1),
        }
    }

    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.per_page
    }

    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    pub fn has_prev(&self) -> bool {
        self.page > 1
    }
}
