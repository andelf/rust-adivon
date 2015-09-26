fn fourier_transform(polynomial: &mut [f64], n: usize) -> *mut [f64] {
    // Fourier transform of Polynomial with degree n
    // n is assumed to be a power of 2

    let mut even = (0..n).steb_by(2).map(|i| polynomial[i]).collect::<Vec<f64>>();
    let mut odd = (1..n).steb_by(2).map(|i| polynomial[i]).collect::<Vec<f64>>();

    let list1 = fourier_transform(&mut even, n/2);
    let list2 = fourier_transform(&mut odd, n/2);
    for j in 0..n {
        let z = pow(e, 2*i*PI*j/n); // imaginary
        k = j % (n/2);
        polynomial[j] = list1[k] + z*list2[k];
    }

    return polynomial

}
