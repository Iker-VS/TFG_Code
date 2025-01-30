use std::io;
#[derive(Debug)]
enum Player {
    PlayerUno,
    PlayerDos,
}
struct Matrix {
    matrix: [[i32; 7]; 6],
}

impl Matrix {
    fn new() -> Self {
        let matrix: [[i32; 7]; 6] = [[0; 7]; 6];
        Matrix { matrix }
    }

    fn add_element_matrix(&mut self, column: usize, player: &Player) -> Result<(), &str> {
        if column >= 7 {
            return Err("El número está fuera de rango");
        }
        for rows in self.matrix.iter_mut().rev() {
            if rows[column] == 0 {
                match player {
                    &Player::PlayerUno => rows[column] = 1,
                    &Player::PlayerDos => rows[column] = 2,
                }
                return Ok(());
            }
        }

        Err("La columna está llena")
    }
    fn print_matrix(&self) {
        let matrix: &[[i32; 7]; 6] = &self.matrix;
        for rows in matrix {
            println!("{:?}", rows)
        }
        println!();
    }
    fn is_full(&self) -> bool {
        for row in &self.matrix {
            for num in row {
                if *num == 0 {
                    return false;
                }
            }
        }
        true
    }
    fn is_win(&self) -> bool {
        let matrix = &self.matrix;

        // Verificar filas (horizontal)
        for i in 0..6 {
            for j in 0..4 {
                if matrix[i][j] != 0
                    && matrix[i][j] == matrix[i][j + 1]
                    && matrix[i][j] == matrix[i][j + 2]
                    && matrix[i][j] == matrix[i][j + 3]
                {
                    return true;
                }
            }
        }

        // Verificar columnas (vertical)
        for i in 0..3 {
            for j in 0..7 {
                if matrix[i][j] != 0
                    && matrix[i][j] == matrix[i + 1][j]
                    && matrix[i][j] == matrix[i + 2][j]
                    && matrix[i][j] == matrix[i + 3][j]
                {
                    return true;
                }
            }
        }

        // Verificar diagonales ↘
        for i in 0..3 {
            for j in 0..4 {
                if matrix[i][j] != 0
                    && matrix[i][j] == matrix[i + 1][j + 1]
                    && matrix[i][j] == matrix[i + 2][j + 2]
                    && matrix[i][j] == matrix[i + 3][j + 3]
                {
                    return true;
                }
            }
        }

        // Verificar diagonales ↙
        for i in 3..6 {
            for j in 0..4 {
                if matrix[i][j] != 0
                    && matrix[i][j] == matrix[i - 1][j + 1]
                    && matrix[i][j] == matrix[i - 2][j + 2]
                    && matrix[i][j] == matrix[i - 3][j + 3]
                {
                    return true;
                }
            }
        }

        false // Si no se encontró victoria
    }
}
fn main() {
    let mut player: Player = Player::PlayerUno;
    let mut juego: Matrix = Matrix::new();
    loop {
        println!("Inserte un número del 0 al 6 ");
        juego.print_matrix();

        loop {
            let column: usize = ask_input();
            match juego.add_element_matrix(column, &player) {
                Ok(()) => break,
                Err(e) => println!("Error: {}", e),
            }
        }
        if juego.is_full(){
            println!("Empate, Fin del juego");
            break;
        }

        juego.print_matrix();

        if juego.is_win(){
            println!("El jugador {:?} Ha ganado, felizidades", player);
            break;
        }

        match player {
            Player::PlayerUno => player = Player::PlayerDos,
            Player::PlayerDos => player = Player::PlayerUno,
        }
    }
}
fn ask_input() -> usize {
    loop {
        // Bucle infinito hasta que se retorne un valor
        let mut input = String::new();

        println!("Ingrese un número:");
        io::stdin()
            .read_line(&mut input)
            .expect("Error al leer la entrada");

        // Intentar convertir a usize
        match input.trim().parse::<usize>() {
            Ok(n) => {
                return n; // Retorna el número si es válido
            }
            Err(_) => {
                println!("Error: ¡No es un número válido! Intente de nuevo.");
            }
        }
    }
}
