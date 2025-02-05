/* 
EJERCICIO DE PUNTEROS EN RUST

Complete las siguientes tareas:
1. Cree un raw pointer a un i32 desde una variable en el stack
2. Cree una referencia inmutable y otra mutable a un valor flotante
3. Use Box para almacenar un valor en el heap
4. Use Rc para compartir ownership de un string
5. Modifique el valor a través de la referencia mutable y del Box
6. Agregue todos los punteros creados al struct PointerPractice
7. Imprima los resultados como se indica al final

Siga los comentarios TODO en el código.
*/

use std::rc::Rc;

struct PointerPractice<'a> {
    raw_ptr: *const i32,
    immutable_ref: &'a f64,
    mutable_ref: &'a mut f64,
    boxed_value: Box<u8>,
    rc_string: Rc<String>,
}

fn pointer_exercise() -> PointerPractice<'static> {
    // TODO 1: Crear raw pointer desde una variable en el stack
    let stack_var = 42;
    raw_ptr=& stack_var;

    
    // TODO 2: Crear referencia inmutable y mutable
    let float_val = 3.14;
    immutable_ref= &float_val;
    mutable_ref =&float_val;
    
    // TODO 3: Usar Box para almacenar 255 en el heap
    let boxed = Box::new(0u8);
    boxed_value=255;
    
    // TODO 4: Usar Rc para compartir este string
    let shared_str = "Hello pointers!".to_string();
    rc_string= &shared_str;
    
    // TODO 5: Modificar valores a través de referencias y Box
    // (la referencia mutable y el Box deben cambiar sus valores)
    
    PointerPractice {
        raw_ptr: std::ptr::null(),
        immutable_ref: &float_val,
        mutable_ref: &mut 0.0,
        boxed_value: boxed,
        rc_string: Rc::new(shared_str),
    }
}

fn main() {
    let practice = pointer_exercise();
    
    // Zona de impresión (NO MODIFICAR)
    unsafe {
        println!("Raw pointer: {}", *practice.raw_ptr);
    }
    println!("Immutable ref: {}", practice.immutable_ref);
    println!("Mutable ref: {}", practice.mutable_ref);
    println!("Boxed value: {}", practice.boxed_value);
    println!("Rc string: {}, Strong count: {}", 
        practice.rc_string, 
        Rc::strong_count(&practice.rc_string)
    );
}