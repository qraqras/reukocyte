use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use ruby_prism;
use ruby_prism::Visit;

/// Generate a Ruby file with the specified number of methods
fn generate_ruby_source(methods: usize, with_violations: bool) -> Vec<u8> {
    let mut source = String::new();

    source.push_str("# frozen_string_literal: true\n\n");
    source.push_str("class Example\n");

    for i in 0..methods {
        if with_violations && i % 10 == 0 {
            // Add trailing whitespace
            source.push_str(&format!("  def method_{i}  \n"));
        } else {
            source.push_str(&format!("  def method_{i}\n"));
        }
        if with_violations && i % 20 == 0 {
            // Add debugger
            source.push_str("    binding.pry\n");
        }
        source.push_str("    @value = calculate_something\n");
        source.push_str("    process(@value)\n");
        source.push_str("  end\n\n");
    }

    source.push_str("end\n");

    source.into_bytes()
}

fn bench_checker(c: &mut Criterion) {
    let mut group = c.benchmark_group("Checker");

    for methods in [50, 100, 500, 1000] {
        let source_clean = generate_ruby_source(methods, false);
        let source_dirty = generate_ruby_source(methods, true);

        group.throughput(Throughput::Bytes(source_clean.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("clean", methods),
            &source_clean,
            |b, source| {
                b.iter(|| reukocyte_checker::check(black_box(source)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("with_violations", methods),
            &source_dirty,
            |b, source| {
                b.iter(|| reukocyte_checker::check(black_box(source)));
            },
        );
    }

    group.finish();
}

fn bench_real_world_file(c: &mut Criterion) {
    // Simulate a typical Rails controller file
    let source = br#"# frozen_string_literal: true

class UsersController < ApplicationController
  before_action :set_user, only: [:show, :edit, :update, :destroy]
  before_action :authorize_user, only: [:edit, :update, :destroy]

  def index
    @users = User.all.page(params[:page])
  end

  def show
  end

  def new
    @user = User.new
  end

  def edit
  end

  def create
    @user = User.new(user_params)

    if @user.save
      redirect_to @user, notice: 'User was successfully created.'
    else
      render :new, status: :unprocessable_entity
    end
  end

  def update
    if @user.update(user_params)
      redirect_to @user, notice: 'User was successfully updated.'
    else
      render :edit, status: :unprocessable_entity
    end
  end

  def destroy
    @user.destroy
    redirect_to users_url, notice: 'User was successfully destroyed.'
  end

  private

  def set_user
    @user = User.find(params[:id])
  end

  def authorize_user
    authorize @user
  end

  def user_params
    params.require(:user).permit(:name, :email, :password, :password_confirmation)
  end
end
"#;

    c.bench_function("Checker/real_world_controller", |b| {
        b.iter(|| reukocyte_checker::check(black_box(source)));
    });
}

/// Benchmark parse vs rules time breakdown
fn bench_parse_vs_rules(c: &mut Criterion) {
    let mut group = c.benchmark_group("TimeBreakdown");

    for methods in [100, 1000, 5000] {
        let source = generate_ruby_source(methods, true);

        group.throughput(Throughput::Bytes(source.len() as u64));

        // 1. Parse only (no AST access)
        group.bench_with_input(
            BenchmarkId::new("1_parse_only", methods),
            &source,
            |b, source| {
                b.iter(|| {
                    let result = ruby_prism::parse(black_box(source));
                    black_box(result)
                });
            },
        );

        // 2. Parse + AST node access (force AST construction)
        group.bench_with_input(
            BenchmarkId::new("2_parse_and_get_node", methods),
            &source,
            |b, source| {
                b.iter(|| {
                    let result = ruby_prism::parse(black_box(source));
                    let _node = result.node();
                    black_box(result)
                });
            },
        );

        // 3. Parse + AST walk (Visitor pattern)
        group.bench_with_input(
            BenchmarkId::new("3_parse_and_walk", methods),
            &source,
            |b, source| {
                b.iter(|| {
                    let result = ruby_prism::parse(black_box(source));
                    let mut checker = reukocyte_checker::Checker::new(black_box(source));
                    checker.visit(&result.node());
                    black_box(checker)
                });
            },
        );

        // 4. Layout rules only (no parse)
        group.bench_with_input(
            BenchmarkId::new("4_layout_rules_only", methods),
            &source,
            |b, source| {
                b.iter(|| {
                    let mut checker = reukocyte_checker::Checker::new(black_box(source));
                    // Manually run layout rules without parse
                    reukocyte_checker::rules::layout::trailing_whitespace::check(&mut checker);
                    reukocyte_checker::rules::layout::trailing_empty_lines::check(&mut checker);
                    reukocyte_checker::rules::layout::leading_empty_lines::check(&mut checker);
                    reukocyte_checker::rules::layout::empty_lines::check(&mut checker);
                    reukocyte_checker::rules::layout::indentation_style::check(&mut checker);
                    black_box(checker.into_diagnostics())
                });
            },
        );

        // 5. Full check (parse + all rules)
        group.bench_with_input(
            BenchmarkId::new("5_full_check", methods),
            &source,
            |b, source| {
                b.iter(|| reukocyte_checker::check(black_box(source)));
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_checker, bench_real_world_file, bench_parse_vs_rules, bench_layout_breakdown);
criterion_main!(benches);

/// Detailed breakdown of layout rules
fn bench_layout_breakdown(c: &mut Criterion) {
    let mut group = c.benchmark_group("LayoutBreakdown");

    for methods in [1000, 5000] {
        let source = generate_ruby_source(methods, true);
        group.throughput(Throughput::Bytes(source.len() as u64));

        // Single line iteration (baseline)
        group.bench_with_input(
            BenchmarkId::new("0_line_iteration_only", methods),
            &source,
            |b, source| {
                b.iter(|| {
                    let mut count = 0usize;
                    for line in source.split(|&b| b == b'\n') {
                        count += line.len();
                    }
                    black_box(count)
                });
            },
        );

        // TrailingWhitespace only
        group.bench_with_input(
            BenchmarkId::new("1_trailing_whitespace", methods),
            &source,
            |b, source| {
                b.iter(|| {
                    let mut checker = reukocyte_checker::Checker::new(black_box(source));
                    reukocyte_checker::rules::layout::trailing_whitespace::check(&mut checker);
                    black_box(checker.into_diagnostics())
                });
            },
        );

        // EmptyLines only
        group.bench_with_input(
            BenchmarkId::new("2_empty_lines", methods),
            &source,
            |b, source| {
                b.iter(|| {
                    let mut checker = reukocyte_checker::Checker::new(black_box(source));
                    reukocyte_checker::rules::layout::empty_lines::check(&mut checker);
                    black_box(checker.into_diagnostics())
                });
            },
        );

        // IndentationStyle only
        group.bench_with_input(
            BenchmarkId::new("3_indentation_style", methods),
            &source,
            |b, source| {
                b.iter(|| {
                    let mut checker = reukocyte_checker::Checker::new(black_box(source));
                    reukocyte_checker::rules::layout::indentation_style::check(&mut checker);
                    black_box(checker.into_diagnostics())
                });
            },
        );

        // All layout rules
        group.bench_with_input(
            BenchmarkId::new("4_all_layout_rules", methods),
            &source,
            |b, source| {
                b.iter(|| {
                    let mut checker = reukocyte_checker::Checker::new(black_box(source));
                    reukocyte_checker::rules::layout::trailing_whitespace::check(&mut checker);
                    reukocyte_checker::rules::layout::trailing_empty_lines::check(&mut checker);
                    reukocyte_checker::rules::layout::leading_empty_lines::check(&mut checker);
                    reukocyte_checker::rules::layout::empty_lines::check(&mut checker);
                    reukocyte_checker::rules::layout::indentation_style::check(&mut checker);
                    black_box(checker.into_diagnostics())
                });
            },
        );
    }

    group.finish();
}
