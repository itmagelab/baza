class BazaBin < Formula
  version '0.10.0'
  desc "Recursively search directories for a regex pattern."
  homepage "https://github.com/itmagelab/baza"

  if OS.mac?
      url "https://github.com/BurntSushi/baza/releases/download/#{version}/baza-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "32754b4173ac87a7bfffd436d601a49362676eb1841ab33440f2f49c002c8967"
  elsif OS.linux?
      url "https://github.com/BurntSushi/baza/releases/download/#{version}/baza-#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "c76080aa807a339b44139885d77d15ad60ab8cdd2c2fdaf345d0985625bc0f97"
  end

  conflicts_with "baza"

  def install
    bin.install "baza"
  end
end
