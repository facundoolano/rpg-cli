# This helper is used to make the pwd match the tracked internally by the game
function sync_rpg
    builtin cd (rpg-cli pwd)
end

# Use the rpg as the command to do non fs related tasks such as print
# status and buy items.
function rpg
    rpg-cli $argv
    sync_rpg
end

# Try to move the hero to the given destination, and cd match the shell pwd
# to that of the hero's location:
# - the one supplied as parameter, if there weren't any battles
# - the one where the battle took place, if the hero wins
# - the home dir, if the hero dies
function cd_fn
  rpg-cli cd "$argv"
  sync_rpg
end

# Some directories have hidden treasure chests that you can find with ls
function l_fn
  command ls "$argv"
  if test 'count $argv' > 0
      rpg cd -f .
      rpg ls
  end
end

alias 'cd'='cd_fn'
alias 'l'='l_fn'
