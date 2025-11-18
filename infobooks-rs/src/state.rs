use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use uuid::Uuid;
use chrono::NaiveDate;

use crate::models::{User, Book, Loan, LoanStatus};

pub struct AppState {
    pub users: Mutex<HashMap<Uuid, User>>,
    pub books: Mutex<HashMap<Uuid, Book>>,
    pub loans: Mutex<HashMap<Uuid, Loan>>,
}

pub static APP_STATE: Lazy<AppState> = Lazy::new(|| {
    let mut users = HashMap::new();
    let mut books = HashMap::new();
    let mut loans = HashMap::new();

    // Admin seed
    let admin_id = Uuid::new_v4();
    users.insert(admin_id, User {
        id: admin_id,
        name: "Admin InfoBooks".into(),
        cpf: "000.000.000-00".into(),
        password: "adminpass".into(),
        is_admin: true,
        active: true,
    });

    // Users seed
    let miguel_id = Uuid::new_v4();
    users.insert(miguel_id, User {
        id: miguel_id,
        name: "Miguel Silva Santos".into(),
        cpf: "458.632.582-07".into(),
        password: "12345678g".into(),
        is_admin: false,
        active: true,
    });

    let lenitha_id = Uuid::new_v4();
    users.insert(lenitha_id, User {
        id: lenitha_id,
        name: "Lenitha Borges".into(),
        cpf: "098.356.333-04".into(),
        password: "lenitha123".into(),
        is_admin: false,
        active: true,
    });

    // Books seed helper
    macro_rules! add_book {
        ($title:expr, $author:expr, $cat:expr, $year:expr, $qty:expr) => {{
            let id = Uuid::new_v4();
            books.insert(id, Book {
                id,
                title: $title.into(),
                author: $author.into(),
                category: $cat.into(),
                year: $year,
                description: format!("Descrição de {}", $title),
                total_qty: $qty,
                available_qty: $qty,
            });
            id
        }};
    }

    let b_moby = add_book!("Moby Dick", "Herman Melville", "Clássicos", 1851, 2);
    let b_divina = add_book!("A Divina Comédia", "Dante Alighieri", "Clássicos", 1320, 1);
    let b_homem_giz = add_book!("O Homem de Giz", "C. J. Tudor", "Suspense", 2018, 3);
    let b_desassossego = add_book!("O Livro do Desassossego", "Fernando Pessoa", "Filosofia", 1982, 1);
    let b_memorias = add_book!("Memórias Póstumas", "Machado de Assis", "Clássicos", 1881, 4);

    // seed a loan for Miguel (borrowed on a date so you can test overdue/no overdue)
    let loan_id = Uuid::new_v4();
    // borrowed 2025-11-01 for example
    let borrowed_at = NaiveDate::from_ymd_opt(2025, 11, 1).unwrap();
    let due_at = borrowed_at + chrono::Duration::days(14);
    loans.insert(loan_id, Loan {
        id: loan_id,
        user_id: miguel_id,
        book_id: b_moby,
        borrowed_at,
        due_at,
        status: LoanStatus::Borrowed,
    });

    AppState {
        users: Mutex::new(users),
        books: Mutex::new(books),
        loans: Mutex::new(loans),
    }
});
