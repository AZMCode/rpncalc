pub trait IntoError where
    Self: syn::spanned::Spanned
{
    fn into_error(&self, err: impl std::fmt::Display) -> syn::Error {
        syn::Error::new(self.span(), err)
    }
}

impl<S> IntoError for S where
    S: syn::spanned::Spanned
{}