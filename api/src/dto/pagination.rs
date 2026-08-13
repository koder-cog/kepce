use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total_items: u64,
    pub total_pages: u64,
    pub current_page: u64,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl PaginationQuery {
    pub fn page_num(&self) -> u64 {
        if let Some(p) = self.page {
            return p.max(1);
        }
        if let Some(off) = self.offset {
            let lim = self.limit_num();
            return (off / lim) + 1;
        }
        1
    }

    pub fn limit_num(&self) -> u64 {
        // En fazla 100, varsayılan 20
        self.limit.unwrap_or(20).clamp(1, 100)
    }
    
    pub fn offset(&self) -> u64 {
        if let Some(off) = self.offset {
            return off;
        }
        (self.page_num() - 1) * self.limit_num()
    }
}
