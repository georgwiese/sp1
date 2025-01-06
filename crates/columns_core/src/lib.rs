pub trait FlattenFieldsHelper {
    fn flatten_fields() -> Option<Vec<String>>;
}

// Implement FlattenFieldsHelper for arrays of any size
impl<T, const N: usize> FlattenFieldsHelper for [T; N] {
    fn flatten_fields() -> Option<Vec<String>> {
        let mut fields = Vec::new();
        for i in 0..N {
            fields.push(i.to_string());
        }
        Some(fields)
    }
}
