use crate::error::{E, R};

pub(crate) fn email_of(value: &str) -> R<String> {
    let value = value.trim().to_lowercase();
    if value.len() > 254 || !value.contains('@') || value.starts_with('@') || value.ends_with('@') {
        Err(E::Bad("Érvényes e-mail cím szükséges.".into()))
    } else {
        Ok(value)
    }
}

pub(crate) fn role(value: &str) -> R<String> {
    match value {
        "admin" | "user" => Ok(value.into()),
        _ => Err(E::Bad("A role értéke admin vagy user lehet.".into())),
    }
}

pub(crate) fn name(value: Option<String>) -> R<Option<String>> {
    let value = value
        .map(|value| value.trim().into())
        .filter(|value: &String| !value.is_empty());
    if value.as_ref().is_some_and(|value| value.len() > 120) {
        Err(E::Bad("A név legfeljebb 120 karakter lehet.".into()))
    } else {
        Ok(value)
    }
}
