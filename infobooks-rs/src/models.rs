use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDate;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub cpf: String,
    pub password: String,
    pub is_admin: bool,
    pub active: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Book {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub category: String,
    pub year: i32,
    pub description: String,
    pub total_qty: u32,
    pub available_qty: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum LoanStatus {
    Pending,
    Borrowed,
    Returned,
    Overdue,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Loan {
    pub id: Uuid,
    pub user_id: Uuid,
    pub book_id: Uuid,
    pub borrowed_at: NaiveDate,
    pub due_at: NaiveDate,
    pub status: LoanStatus,
}
