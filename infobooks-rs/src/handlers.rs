use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::Deserialize;
use tera::Tera;
use crate::state::AppState;
use crate::models::{Book, User, Loan};
use uuid::Uuid;
use chrono::{Utc, Duration, NaiveDate};

#[derive(Deserialize)]
pub struct LoginForm {
    cpf: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterForm {
    name: String,
    cpf: String,
    password: String,
}

pub async fn index(tera: web::Data<Tera>, data: web::Data<AppState>) -> impl Responder {
    let books = data.books.read().clone();
    let ctx = {
        let mut t = tera::Context::new();
        t.insert("books", &books);
        t
    };
    let body = tera.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

pub async fn login_page(tera: web::Data<Tera>) -> impl Responder {
    let body = tera.render("login.html", &tera::Context::new()).unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

pub async fn do_login(form: web::Form<LoginForm>, data: web::Data<AppState>) -> impl Responder {
    // very simple auth: check cpf/password
    let cpf = form.cpf.trim();
    if let Some(user) = data.find_user(cpf) {
        if user.password == form.password {
            // in ausência de sessão, retornamos JSON com sucesso e role
            return HttpResponse::Ok().json(serde_json::json!({
                "status": "ok",
                "name": user.name,
                "cpf": user.cpf,
                "is_admin": user.is_admin
            }));
        }
    }
    HttpResponse::Unauthorized().json(serde_json::json!({"status":"unauthorized"}))
}

pub async fn register_page(tera: web::Data<Tera>) -> impl Responder {
    let body = tera.render("register.html", &tera::Context::new()).unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

pub async fn do_register(form: web::Form<RegisterForm>, data: web::Data<AppState>) -> impl Responder {
    // basic validation: cpf unique
    let cpf = form.cpf.trim().to_string();
    if data.find_user(&cpf).is_some() {
        return HttpResponse::BadRequest().json(serde_json::json!({"status":"cpf_exists"}));
    }
    let user = User {
        name: form.name.clone(),
        cpf: cpf.clone(),
        password: form.password.clone(),
        is_admin: false,
    };
    data.users.write().push(user);
    HttpResponse::Ok().json(serde_json::json!({"status":"registered","cpf":cpf}))
}

pub async fn book_page(path: web::Path<(String,)>, tera: web::Data<Tera>, data: web::Data<AppState>) -> impl Responder {
    let id_str = &path.0;
    let id = match Uuid::parse_str(id_str) {
        Ok(u) => u,
        Err(_) => return HttpResponse::BadRequest().body("Invalid book id"),
    };
    if let Some(book) = data.find_book(&id) {
        let mut ctx = tera::Context::new();
        ctx.insert("book", &book);
        let body = tera.render("book.html", &ctx).unwrap();
        return HttpResponse::Ok().content_type("text/html").body(body);
    }
    HttpResponse::NotFound().body("Book not found")
}

#[derive(Deserialize)]
pub struct RentRequest {
    user_cpf: String,
    book_id: String,
}

pub async fn rent_book(form: web::Form<RentRequest>, data: web::Data<AppState>) -> impl Responder {
    let cpf = form.user_cpf.trim().to_string();
    let book_id = match Uuid::parse_str(&form.book_id) {
        Ok(u) => u,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"status":"invalid_book"})),
    };
    let mut books = data.books.write();
    if let Some(book) = books.iter_mut().find(|b| b.id==book_id) {
        if book.available_quantity == 0 {
            return HttpResponse::BadRequest().json(serde_json::json!({"status":"no_available"}));
        }
        // create loan
        let rent_date = Utc::now().date_naive();
        let due_date = rent_date + Duration::days(14);
        let loan = Loan {
            id: Uuid::new_v4(),
            user_cpf: cpf.clone(),
            book_id,
            rent_date,
            due_date,
            returned: false,
            return_date: None,
        };
        data.loans.write().push(loan.clone());
        book.available_quantity -= 1;
        return HttpResponse::Ok().json(serde_json::json!({"status":"rented","loan_id": loan.id.to_string(), "due_date": due_date.to_string()}));
    }
    HttpResponse::NotFound().json(serde_json::json!({"status":"book_not_found"}))
}

#[derive(Deserialize)]
pub struct ReturnRequest {
    user_cpf: String,
    loan_id: String,
}

pub async fn return_book(form: web::Form<ReturnRequest>, data: web::Data<AppState>) -> impl Responder {
    let cpf = form.user_cpf.trim();
    let loan_id = match Uuid::parse_str(&form.loan_id) {
        Ok(u) => u,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"status":"invalid_loan"})),
    };

    let mut loans = data.loans.write();
    if let Some(loan) = loans.iter_mut().find(|l| l.id==loan_id && l.user_cpf==cpf) {
        if loan.returned {
            return HttpResponse::BadRequest().json(serde_json::json!({"status":"already_returned"}));
        }
        let today = Utc::now().date_naive();
        if today > loan.due_date {
            // overdue -> cannot auto-devolve (segundo sua descrição)
            return HttpResponse::BadRequest().json(serde_json::json!({"status":"overdue","message":"Procure atendimento administrativo"}));
        }
        loan.returned = true;
        loan.return_date = Some(today);
        // increase book availability
        if let Some(mut book) = data.books.write().iter_mut().find(|b| b.id==loan.book_id) {
            book.available_quantity = book.available_quantity.saturating_add(1);
        }
        return HttpResponse::Ok().json(serde_json::json!({"status":"returned"}));
    }
    HttpResponse::NotFound().json(serde_json::json!({"status":"loan_not_found"}))
}

// Admin endpoints (simple)
pub async fn admin_dashboard(tera: web::Data<Tera>, data: web::Data<AppState>) -> impl Responder {
    let books = data.books.read().clone();
    let loans = data.loans.read().clone();
    let users = data.users.read().clone();
    let total_books = books.len();
    let total_loans = loans.iter().filter(|l| !l.returned).count();
    let active_users = users.len();
    let overdue = loans.iter().filter(|l| !l.returned && l.due_date < Utc::now().date_naive()).count();

    let mut ctx = tera::Context::new();
    ctx.insert("total_books", &total_books);
    ctx.insert("total_loans", &total_loans);
    ctx.insert("active_users", &active_users);
    ctx.insert("overdue", &overdue);
    let body = tera.render("admin_dashboard.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

// Additional handlers for admin book/user lists could be added similarly.
