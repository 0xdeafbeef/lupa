function activate --argument-names root
    set -l bin (path join $root bin)
    if test -d $bin
        fish_add_path $bin
    end
    for file in (command ls $bin)
        echo $file
    end
end

function deactivate
    if set -q _OLD_PATH
        set PATH $_OLD_PATH
    end
end
