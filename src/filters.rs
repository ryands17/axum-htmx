pub fn yes_no<T: std::fmt::Display>(condition: &bool, yes: T, no: T) -> ::askama::Result<String> {
  if *condition {
    Ok(yes.to_string())
  } else {
    Ok(no.to_string())
  }
}
