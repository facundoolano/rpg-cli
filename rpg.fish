set rpg ~/dev/facundoolano/rpg-cli/target/release/rpg-cli
$rpg $argv
set dest ($rpg --pwd)
cd $dest
