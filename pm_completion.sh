
_pm_completions() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    opts="open edit add remove add-source list"

    if [[ ${COMP_CWORD} == 1 ]]; then
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi

        case "${prev}" in
            open|remove)
                local projects=$(pm list | awk '{print $1}')
                COMPREPLY=( $(compgen -W "${projects}" -- ${cur}) )
                return 0
                ;;
            *)
                ;;
        esac
}

pmcd() {
    cd "$(pm pwd "$1")"
}

_pmcd_completions() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    opts=$(pm list | awk '{print $1}')

    COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
    return 0
}

complete -F _pmcd_completions pmcd
complete -F _pm_completions pm