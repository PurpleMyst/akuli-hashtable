use std::{
    collections::hash_map::RandomState, hash::{BuildHasher, Hash, Hasher}, marker::PhantomData,
    mem, os::raw::{c_int, c_uint, c_void}, ptr::{self, NonNull},
};

use super::error::HashTableError;

use hashtable_sys as sys;

// TODO: Explain why I do this.
const STATUS_OK: c_int = sys::STATUS_OK as c_int;

#[derive(Debug)]
pub struct HashTable<K: Eq + Hash, V> {
    ptr: NonNull<sys::HashTable>,

    keys: Vec<(K, c_uint)>,
    values: Vec<V>,

    _phantom_key: PhantomData<K>,
    _phantom_value: PhantomData<V>,
}

impl<K: Eq + Hash, V> HashTable<K, V> {
    pub fn new() -> Option<Self> {
        #[allow(dead_code)]
        extern "C" fn cmp_func<T: Eq>(
            a: *mut c_void,
            b: *mut c_void,
            _userdata: *mut c_void,
        ) -> c_int {
            let (a, b) = (a as *mut T, b as *mut T);

            unsafe {
                match (*a) == (*b) {
                    true => 1,
                    false => 0,
                }
            }
        };

        let maybe_ptr = NonNull::new(unsafe { sys::hashtable_new(Some(cmp_func::<K>)) });

        maybe_ptr.map(|ptr| Self {
            ptr,

            keys: Default::default(),
            values: Default::default(),

            _phantom_key: PhantomData,
            _phantom_value: PhantomData,
        })
    }

    // return STATUS_OK on success, and on failure STATUS_NOMEM or an error code from cmpfunc
    //int hashtable_set(struct HashTable *ht,
    //                  void *key,
    //                  unsigned int keyhash,
    //                  void *value,
    //                  void *userdata);
    pub fn set(&mut self, key: K, value: V) -> Result<(), HashTableError> {
        let random_state = RandomState::new();
        let mut hasher = random_state.build_hasher();
        key.hash(&mut hasher);
        let keyhash = (hasher.finish() % (c_uint::max_value() as u64 + 1)) as c_uint;

        self.keys.push((key, keyhash));
        self.values.push(value);

        let status = unsafe {
            sys::hashtable_set(
                self.ptr.as_ptr(),
                &mut self.keys.last_mut().unwrap().0 as *mut K as *mut c_void,
                keyhash,
                self.values.last_mut().unwrap() as *mut V as *mut c_void,
                ptr::null_mut(),
            )
        };

        match status {
            STATUS_OK => Ok(()),
            sys::STATUS_NOMEM => Err(HashTableError::NoMem),

            _ => unreachable!(),
        }
    }

    // return 1 if key was found, 0 if not (*res is not set) or an error code from cmpfunc
    //int hashtable_get(struct HashTable *ht,
    //                  void *key,
    //                  unsigned int keyhash,
    //                  void **res,
    //                  void *userdata);
    pub fn get(&mut self, needle: &K) -> Option<&mut V>
    where
        V: Eq,
    {
        let (key, keyhash) = self.keys.iter().find(|(key, _keyhash)| key == needle)?;

        unsafe {
            let mut res_inner = mem::uninitialized::<*mut c_void>();

            let key_was_found = sys::hashtable_get(
                self.ptr.as_ptr(),
                key as *const K as *mut c_void,
                *keyhash,
                &mut res_inner as *mut *mut c_void,
                ptr::null_mut(),
            );

            match key_was_found {
                0 => None,
                1 => {
                    let needle = res_inner as *mut V;
                    self.values.iter_mut().find(|value| **value == *needle)
                }

                _ => unreachable!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn can_retrieve<K: Eq + Hash + Clone, V: Eq + Clone>(key: K, value: V) -> bool {
        let mut hash_table = HashTable::new().unwrap();
        hash_table.set(key.clone(), value.clone()).unwrap();

        *hash_table.get(&key).unwrap() == value
    }

    macro_rules! test_can_retrieve {
        ($name:ident => $key_ty:ty, $value_ty:ty) => {
            #[quickcheck]
            fn $name(key: $key_ty, value: $value_ty) -> bool {
                can_retrieve(key, value)
            }
        };
    }

    test_can_retrieve!(can_retrieve_string_string => String, String);
    test_can_retrieve!(can_retrieve_string_u8 => String, u8);
    test_can_retrieve!(can_retrieve_string_u32 => String, u32);
    test_can_retrieve!(can_retrieve_usize_string => usize, String);
    test_can_retrieve!(can_retrieve_u8_string => u8, String);
    test_can_retrieve!(can_retrieve_vecstring_vecvecu8 => Vec<String>, Vec<Vec<u8>>);

    #[test]
    fn growing_works() {
        const ELEMENTS: usize = 51;

        let mut hash_table = HashTable::<usize, usize>::new().unwrap();

        for i in 0..ELEMENTS {
            hash_table.set(0xDEAD + i, 0xBEEF + i).unwrap();
        }

        for i in 0..ELEMENTS {
            assert_eq!(*hash_table.get(&(0xDEAD + i)).unwrap(), 0xBEEF + i);
        }
    }
}
