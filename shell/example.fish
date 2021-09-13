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
function cd
    if count $argv > /dev/null
        rpg-cli cd "$argv"
    else
        rpg-cli cd $HOME
    end
    sync_rpg
end

# Some directories have hidden treasure chests that you can find with ls
function ls
    if count $argv > /dev/null
        rpg-cli cd -f $argv[1]
        rpg-cli ls
        command ls $argv[1]
        rpg-cli cd -f (pwd)
    else
        rpg-cli ls
        command ls
    end
end

# Create dungeon
function dn
    set regex '^[0-9]+$'
    set location (basename (pwd))
    if string match -r -q $regex $location
        set next_dir (math $location + 1)
        command mkdir -p $next_dir && cd $next_dir && rpg ls
    else if string match -r -q '^dungeon$' $location
        command mkdir -p 1 && cd 1 && rpg ls
    else
        command mkdir -p dungeon && rpg-cli cd dungeon && rpg ls
    end
end