pub fn cpf_normalize(cpf: &str) -> String {
    cpf.chars().filter(|c| c.is_digit(10)).collect()
}
