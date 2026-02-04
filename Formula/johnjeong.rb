class Johnjeong < Formula
  desc "Terminal edition of johnjeong.com"
  homepage "https://johnjeong.com"
  url "https://github.com/ComputelessComputer/homebrew-johnjeong/archive/refs/tags/v0.1.3.tar.gz"
  sha256 "deba0e4e2429e928346ce8cd502a01c9cfb2514d5277a01a67e9b2220b0a995d"
  license "MIT"
  depends_on "git"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "cli")
  end

  test do
    system "#{bin}/johnjeong", "--help"
  end
end
