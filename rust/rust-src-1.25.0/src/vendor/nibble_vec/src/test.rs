use NibbleVec;

fn v8_7_6_5() -> NibbleVec {
    NibbleVec::from_byte_vec(vec![8 << 4 | 7, 6 << 4 | 5])
}

fn v11_10_9() -> NibbleVec {
    let mut result = NibbleVec::from_byte_vec(vec![11 << 4 | 10]);
    result.push(9);
    result
}

#[test]
fn get() {
    let nv = NibbleVec::from_byte_vec(vec![3 << 4 | 7]);
    assert_eq!(nv.get(0), 3u8);
    assert_eq!(nv.get(1), 7u8);
}

#[test]
fn push() {
    let mut nv = NibbleVec::new();
    let data = vec![0, 1, 3, 5, 7, 9, 11, 15];
    for val in data.iter() {
        nv.push(*val);
    }

    for (i, val) in data.iter().enumerate() {
        assert_eq!(nv.get(i), *val);
    }
}

fn split_test(  nibble_vec: &NibbleVec,
                idx: usize,
                first: Vec<u8>,
                second: Vec<u8>) {
    let mut init = nibble_vec.clone();
    let tail = init.split(idx);
    assert!(init == first[..]);
    assert!(tail == second[..]);
}

#[test]
fn split_even_length() {
    let even_length = v8_7_6_5();
    split_test(&even_length, 0, vec![], vec![8, 7, 6, 5]);
    split_test(&even_length, 1, vec![8], vec![7, 6, 5]);
    split_test(&even_length, 2, vec![8, 7], vec![6, 5]);
    split_test(&even_length, 4, vec![8, 7, 6, 5], vec![]);
}

#[test]
fn split_odd_length() {
    let odd_length = v11_10_9();
    split_test(&odd_length, 0, vec![], vec![11, 10, 9]);
    split_test(&odd_length, 1, vec![11], vec![10, 9]);
    split_test(&odd_length, 2, vec![11, 10], vec![9]);
    split_test(&odd_length, 3, vec![11, 10, 9], vec![]);
}

/// Join vec2 onto vec1 and ensure that the results matches the one expected.
fn join_test(vec1: &NibbleVec, vec2: &NibbleVec, result: Vec<u8>) {
    let joined = vec1.clone().join(vec2);
    assert!(joined == result[..]);
}

#[test]
fn join_even_length() {
    let v1 = v8_7_6_5();
    let v2 = v11_10_9();
    join_test(&v1, &v2, vec![8, 7, 6, 5, 11, 10, 9]);
    join_test(&v1, &v1, vec![8, 7, 6, 5, 8, 7, 6, 5]);
    join_test(&v1, &NibbleVec::new(), vec![8, 7, 6, 5]);
    join_test(&NibbleVec::new(), &v1, vec![8, 7, 6, 5]);
}

#[test]
fn join_odd_length() {
    let v1 = v8_7_6_5();
    let v2 = v11_10_9();
    join_test(&v2, &v1, vec![11, 10, 9, 8, 7, 6, 5]);
    join_test(&v2, &v2, vec![11, 10, 9, 11, 10, 9]);
    join_test(&v2, &NibbleVec::new(), vec![11, 10, 9]);
}

#[test]
fn clone() {
    let v1 = v8_7_6_5().clone();
    assert_eq!(v1.len(), 4);
}

/// Ensure that the last nibble is zeroed before reuse.
#[test]
fn memory_reuse() {
    let mut vec = NibbleVec::new();
    vec.push(10);
    vec.push(1);

    // Pushing.
    vec.split(1);
    vec.push(2);
    assert_eq!(vec.get(1), 2);

    // Joining.
    vec.split(1);
    vec = vec.join(&NibbleVec::from_byte_vec(vec![1 << 4 | 3, 5 << 4]));
    assert_eq!(vec.get(1), 1);
}
