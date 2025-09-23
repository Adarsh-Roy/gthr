class Gthr < Formula
  desc "A CLI tool for interactive directory text ingestion with fuzzy finder"
  homepage "https://github.com/Adarsh-Roy/gthr"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Adarsh-Roy/gthr/releases/download/v0.1.0/gthr-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_ARM64"
    else
      url "https://github.com/Adarsh-Roy/gthr/releases/download/v0.1.0/gthr-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_X86_64"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/Adarsh-Roy/gthr/releases/download/v0.1.0/gthr-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_LINUX"
    end
  end

  def install
    bin.install "gthr-aarch64-apple-darwin" => "gthr" if Hardware::CPU.arm? && OS.mac?
    bin.install "gthr-x86_64-apple-darwin" => "gthr" if Hardware::CPU.intel? && OS.mac?
    bin.install "gthr-x86_64-unknown-linux-gnu" => "gthr" if OS.linux?
  end

  test do
    system "#{bin}/gthr", "--version"
    system "#{bin}/gthr", "--help"
  end
end