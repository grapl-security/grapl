use fnv::FnvHashMap as HashMap;

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
            String::with_capacity(2 * self.variables.len() + 9 * self.variables.len());
        for variable_name in self.variables.values() {
            variables.push_str(variable_name);
            variables.push_str(":String!,");
        }
        variables
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

            String::from_utf8(self.variable.clone()).unwrap()
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
    }
}
