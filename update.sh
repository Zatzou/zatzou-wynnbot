git pull
cargo build --release
rm ./zatzoubot
cp ./target/release/zatzoubot ./zatzoubot
pkill zatzoubot
screen -d -m ./zatzoubot
screen -ls
