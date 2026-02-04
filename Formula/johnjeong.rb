class Johnjeong < Formula
  desc "Terminal edition of johnjeong.com"
  homepage "https://johnjeong.com"
  url "https://github.com/ComputelessComputer/homebrew-johnjeong/archive/refs/tags/v0.1.2.tar.gz"
  sha256 "3ae884eafbb04d12059935d8329f312ca0b946c070136ece012d91fd6b118ef9"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "cli")
  end

  test do
    system "#{bin}/johnjeong", "--help"
  end
end
