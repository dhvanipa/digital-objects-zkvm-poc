use sp1_build::build_program_with_args;

fn main() {
    build_program_with_args("../pow", Default::default());
    build_program_with_args("../stone", Default::default());
    build_program_with_args("../wood", Default::default());
    build_program_with_args("../axe", Default::default());
}
