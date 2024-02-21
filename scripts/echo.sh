source ${BASH_SOURCE%/*}/setenv.sh
printenv MAELSTROM_LOC
$MAELSTROM_LOC/maelstrom test -w echo --bin ../target/debug/echo --node-count 1 --time-limit 10
