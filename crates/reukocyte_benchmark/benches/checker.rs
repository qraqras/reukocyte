use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

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

criterion_group!(benches, bench_checker, bench_real_world_file);
criterion_main!(benches);
