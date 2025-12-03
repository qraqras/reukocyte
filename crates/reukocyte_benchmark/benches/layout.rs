use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

/// Generate a Ruby file with the specified number of lines
fn generate_ruby_source(lines: usize, with_violations: bool) -> Vec<u8> {
    let mut source = String::new();

    source.push_str("# frozen_string_literal: true\n\n");
    source.push_str("class Example\n");

    for i in 0..lines {
        if with_violations && i % 10 == 0 {
            // Add trailing whitespace every 10 lines
            source.push_str(&format!("  def method_{i}  \n"));
        } else {
            source.push_str(&format!("  def method_{i}\n"));
        }
        source.push_str("    @value = 42\n");
        source.push_str("  end\n\n");
    }

    source.push_str("end\n");

    source.into_bytes()
}

fn bench_trailing_whitespace(c: &mut Criterion) {
    let mut group = c.benchmark_group("Layout/TrailingWhitespace");

    for lines in [100, 500, 1000, 5000] {
        let source_clean = generate_ruby_source(lines, false);
        let source_dirty = generate_ruby_source(lines, true);

        group.throughput(Throughput::Bytes(source_clean.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("clean", lines),
            &source_clean,
            |b, source| {
                b.iter(|| reukocyte_checker::check(black_box(source)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("with_violations", lines),
            &source_dirty,
            |b, source| {
                b.iter(|| reukocyte_checker::check(black_box(source)));
            },
        );
    }

    group.finish();
}

fn bench_real_world_file(c: &mut Criterion) {
    // Simulate a typical Rails model file
    let source = br#"# frozen_string_literal: true

class User < ApplicationRecord
  belongs_to :organization
  has_many :posts, dependent: :destroy
  has_many :comments, dependent: :destroy

  validates :email, presence: true, uniqueness: true
  validates :name, presence: true, length: { minimum: 2, maximum: 100 }

  scope :active, -> { where(active: true) }
  scope :admins, -> { where(admin: true) }

  before_save :normalize_email
  after_create :send_welcome_email

  def full_name
    [first_name, last_name].join(' ')
  end

  def admin?
    role == 'admin'
  end

  def activate!
    update!(active: true, activated_at: Time.current)
  end

  private

  def normalize_email
    self.email = email.downcase.strip
  end

  def send_welcome_email
    UserMailer.welcome(self).deliver_later
  end
end
"#;

    c.bench_function("Layout/real_world_model", |b| {
        b.iter(|| reukocyte_checker::check(black_box(source)));
    });
}

criterion_group!(benches, bench_trailing_whitespace, bench_real_world_file);
criterion_main!(benches);
