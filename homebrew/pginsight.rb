class Pginsight < Formula
  desc "Interactive terminal UI for monitoring PostgreSQL"
  homepage "https://github.com/eliemugenzi/pginsight"
  version "0.1.0"
  license "MIT"

  on_macos do
    url "https://github.com/eliemugenzi/pginsight/releases/download/v#{version}/pginsight-v#{version}-macos-universal.tar.gz"
    sha256 "REPLACE_WITH_SHA256_AFTER_FIRST_RELEASE"
  end

  def install
    bin.install "pginsight"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/pginsight --version")
  end
end
