#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn major_test() {
        let command = "vg list";
        let parts = command.split(' ').collect::<Vec<&str>>();

        if let Some((_,)) = gpt!("vg", "kill"; &parts => u128) {
            panic!()
        } else if gpt!("vg", "list"; &parts => ).is_some() {
        } else {
            panic!()
        }
    }

    #[test]
    fn test_name() {
        assert_eq!(map_type!(["salam"] => u32), None);
        assert_eq!(map_type!(["10"] => u32), Some((10,)));
        assert_eq!(map_type!(["10"] => String, u32), None);
        assert_eq!(
            map_type!(["count", "10"] => String, u32),
            Some(("count".to_string(), 10))
        );
    }

    #[test]
    fn test_2() {
        if let Some((arg1, arg2)) = map_type!(["count", "20"] => String, u32) {
            assert_eq!(arg1, "count".to_string());
            assert_eq!(arg2, 20);
        } else {
            panic!()
        }
    }

    #[test]
    fn test_3() {
        if let Some((arg1,)) = gpt!("count"; ["count", "20"] => u32) {
            assert_eq!(arg1, 20);
        } else {
            panic!()
        }
    }

    #[test]
    fn test_4() {
        if let Some((vg_id,)) = gpt!("vg", "kill"; ["vg", "kill", "123456"] => u128) {
            assert_eq!(vg_id, 123456)
        } else {
            panic!()
        }
    }

    #[test]
    fn test_5() {
        if let Some((a1, a2, vg_id)) = gpt!(; ["vg", "kill", "123456"] => String, String, u128) {
            assert_eq!(a1, "vg".to_string());
            assert_eq!(a2, "kill".to_string());
            assert_eq!(vg_id, 123456)
        } else {
            panic!()
        }
    }
}

/// Guarded map type
#[macro_export]
macro_rules! gpt {
    ($( $guard:expr ),* $(,)? ; $exp:expr => $( $type:ty ),* $(,)?) => {
        {
            let mut iterable = $exp.iter();
            #[allow(unused_mut)]
            let mut should_guards: Vec<String> = vec![];
            #[allow(unused_mut)]
            let mut guards: Vec<String> = vec![];

            $(
                match iterable.next() {
                    Some(item) => should_guards.push(item.to_string()),
                    None => (),
                };

                guards.push($guard.to_string());
            )*

            if should_guards != guards {
                None
            } else {
                #[allow(unused_mut)]
                let mut has_fails = false;
                let result = (
                    $(
                        match iterable.next() {
                            Some(item) => match item.parse::<$type>() {
                                Ok(item) => item,
                                Err(_) => {
                                    has_fails = true;
                                    Default::default()
                                },
                            },
                            None => {
                                has_fails = true;
                                Default::default()
                            },
                        },
                    )*
                );

                if has_fails {
                    None
                } else {
                    Some(result)
                }
            }
        }
    };
}

#[macro_export]
macro_rules! map_type {
    ($exp:expr => $( $type:ty ),+ $(,)?) => {
        {
            let mut iterable = $exp.into_iter();
            let mut has_fails = false;

            let result = (
                $(
                    match iterable.next() {
                        Some(item) => match item.parse::<$type>() {
                            Ok(item) => item,
                            Err(_) => {
                                has_fails = true;
                                Default::default()
                            },
                        },
                        None => {
                            has_fails = true;
                            Default::default()
                        },
                    },
                )*
            );

            if has_fails {
                None
            } else {
                Some(result)
            }
        }
    };
}

pub use gpt;
pub use map_type;
