#[macro_export]
macro_rules! error_chain {
        (
         $(#[$meta:meta])*
         $vis:vis enum $enum_name:ident {
             $(
                $(#[$entry_meta:meta])*
                $entry_name:ident($($(#[$input_meta:meta])* $input_type:ty),*)
                ),*$(,)+
        }
        ) => {
                $(#[$meta])*
                $vis enum $enum_name{
                    $(
                    $(#[$entry_meta])*
                    $entry_name($($(#[$input_meta])* $input_type),*),
                    )*
                }
                impl std::fmt::Debug for $enum_name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        use std::error::Error;
                        writeln!(f, "{}\n", self)?;
                        let mut current = self.source();
                        while let Some(cause) = current {
                            writeln!(f, "Caused by:\n\t{}", cause)?;
                            current = cause.source();
                        }
                        Ok(())
                    }
                }
        }
    }
