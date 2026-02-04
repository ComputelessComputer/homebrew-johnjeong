class Johnjeong < Formula
  desc "Terminal edition of johnjeong.com"
  homepage "https://johnjeong.com"
  url "https://github.com/ComputelessComputer/homebrew-johnjeong/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "b623f9cf7b37edb2cbc88c85f9cf86c543b363c9e414a5968dda9af83e08e796"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "cli")
  end

  test do
    system "#{bin}/johnjeong", "--help"
  end
end
