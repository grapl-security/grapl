use fnv::FnvHashMap as HashMap;

/// VarAllocator allocates graphql variables, one for each value to be interpolated
/// The values will be sequentially defined from $a to $z, wrapping to $za
/// GraphQL variables prevent injection vulnerabilities since the variables are
/// sent as json, separate from the query itself.
#[derive(Clone)]
pub struct VarAllocator {
    variables: HashMap<String, String>,
    variable: Vec<u8>,
    last_var: u8,
}

impl Default for VarAllocator {
    fn default() -> Self {
        Self {
            variables: HashMap::default(),
            variable: b"$".to_vec(),
            last_var: b'z',
        }
    }
}

impl VarAllocator {
    pub(crate) fn variable_string(&self) -> String {
        let mut variables =
            String::with_capacity((2 * self.variables.len()) + (8 * self.variables.len()));
        for (i, variable_name) in self.variables.values().enumerate() {
            variables.push_str(variable_name);
            variables.push_str(":string");
            if i < self.variables.len() - 1 {
                variables.push(',');
            }
        }
        variables
    }

    pub fn variable_map(self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::with_capacity(self.variables.len());
        for (k, v) in self.variables.into_iter() {
            map.insert(v, k);
        }
        map
    }

    pub fn alloc(&mut self, value: String) -> &str {
        self.variables.entry(value).or_insert_with(|| {
            if self.last_var == b'z' {
                self.last_var = b'a';
                self.variable.push(self.last_var);
            } else {
                self.last_var += 1;
                *self.variable.last_mut().unwrap() = self.last_var;
            }

            // It's guaranteed ascii, which is guaranteed utf8
            unsafe {
                debug_assert!(String::from_utf8(self.variable.clone()).is_ok());
                String::from_utf8_unchecked(self.variable.clone())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_alloc() {
        const A: u8 = b'a';
        const Z: u8 = b'z';

        let mut allocator = VarAllocator::default();

        for i in A..Z {
            let var = allocator.alloc(i.to_string());
            assert_eq!(var, format!("${}", i as char));
        }
        let var = allocator.alloc("abcd".into());
        assert_eq!(var, "$z");
        let var = allocator.alloc("efgh".into());
        assert_eq!(var, "$za");
    }
}
