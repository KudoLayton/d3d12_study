use directx_math::*;

fn vector_to_string(vector: &FXMVECTOR) -> String
{
    let mut readable_vector = XMFLOAT4{
        x: 0.0, 
        y: 0.0, 
        z: 0.0, 
        w: 0.0
    };
    XMStoreFloat4(&mut readable_vector, *vector);
    format!("({}, {}, {}, {})",readable_vector.x, readable_vector.y, readable_vector.z, readable_vector.w)
}

fn main() {
    let p = XMVectorZero();
    let q = XMVectorSplatOne();
    let u = XMVectorSet(1.0, 2.0, 3.0, 0.0);
    let v = XMVectorReplicate(-2.0);
    let w = XMVectorSplatZ(u);

    println!("p = {}", vector_to_string(&p));
    println!("q = {}", vector_to_string(&q));
    println!("u = {}", vector_to_string(&u));
    println!("v = {}", vector_to_string(&v));
    println!("w = {}", vector_to_string(&w));
}
