mod symbolic_builder;
mod symbolic_expression;
mod symbolic_variable;

use core::panic;

use p3_field::Field;
pub use symbolic_builder::*;
use symbolic_expression::SymbolicExpression;
use symbolic_variable::{Entry, SymbolicVariable};

pub fn get_pil<F: Field>(columns: Vec<String>, ab: SymbolicAirBuilder<F>) -> String {
    let mut pil = "
col fixed is_first_row = [1] + [0]*;
col fixed is_last_row = [0] + [1]*;
col fixed is_transition = [0] + [1]* + [0];
"
    .to_string();

    // Declare witness columns
    for column in &columns {
        pil.push_str(&format!("col witness {column};\n"));
    }

    for constraint in &ab.constraints {
        // println!("{}", format_expr(constraint, &columns));
        pil.push_str(&format!("{} = 0;\n", format_expr(constraint, &columns)));
    }
    pil
}

fn format_expr<F: Field>(expr: &SymbolicExpression<F>, columns: &[String]) -> String {
    match expr {
        SymbolicExpression::Variable(SymbolicVariable { entry, index, _phantom }) => {
            let offset_str = |offset| match offset {
                0 => "",
                1 => "'",
                _ => unimplemented!(),
            };
            match entry {
                Entry::Preprocessed { .. } => {
                    unimplemented!()
                }
                Entry::Main { offset } => {
                    let column_name = columns.get(*index).unwrap_or_else(|| {
                        panic!("Column index out of bounds: {}\nColumns: {:?}", index, columns)
                    });
                    format!("{column_name}{}", offset_str(*offset))
                }
                Entry::Permutation { .. } => unimplemented!(),
                Entry::Public => format!(":public_{index}"),
                Entry::Challenge => unimplemented!(),
            }
        }
        SymbolicExpression::IsFirstRow => "is_first_row".to_string(),
        SymbolicExpression::IsLastRow => "is_last_row".to_string(),
        SymbolicExpression::IsTransition => "is_transition".to_string(),
        SymbolicExpression::Constant(c) => format!("{}", c),
        SymbolicExpression::Add { x, y, degree_multiple } => {
            format!("({} + {})", format_expr(x, columns), format_expr(y, columns))
        }
        SymbolicExpression::Sub { x, y, degree_multiple } => {
            format!("({} - {})", format_expr(x, columns), format_expr(y, columns))
        }
        SymbolicExpression::Neg { x, degree_multiple } => format!("(-{})", format_expr(x, columns)),
        SymbolicExpression::Mul { x, y, degree_multiple } => {
            format!("({} * {})", format_expr(x, columns), format_expr(y, columns))
        }
    }
}
