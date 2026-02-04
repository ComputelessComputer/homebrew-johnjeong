class Johnjeong < Formula
  desc "Terminal edition of johnjeong.com"
  homepage "https://johnjeong.com"
  url "https://github.com/ComputelessComputer/homebrew-johnjeong/archive/refs/tags/v0.1.1.tar.gz"
  sha256 "2ee386c855ae918d68ba9f78c3b2e9e7f77bf4f4c08b809451b318c82223c2ad"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "cli")
  end

  test do
    system "#{bin}/johnjeong", "--help"
  end
end
