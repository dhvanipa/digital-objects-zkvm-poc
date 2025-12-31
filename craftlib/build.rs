use sp1_build::build_program_with_args;

fn main() {
    build_program_with_args("../programs/pow", Default::default());
    build_program_with_args("../programs/wood", Default::default());
    build_program_with_args("../programs/stone", Default::default());
    build_program_with_args("../programs/axe", Default::default());
}
