# Install Git shell completion.
source /usr/share/git-core/contrib/completion/git-prompt.sh

# Cache terminal color codes for the prompt.
# \x01 and \x02 (or \[ and \]) are open/close tags for
# "non-printing" characters; we want BASH
# to know that these color codes don't get
# rendered on-screen, or else things like the
# history command will mangle the prompt.
GREEN=\\x01$(tput setaf 2)\\x02
MAGENTA=\\x01$(tput setaf 198)\\x02
MAGENTA_B=\\[$(tput setaf 198)\\]
PURPLE=\\x01$(tput setaf 5)\\x02
FAINT=\\x01$(tput setaf 255)\\x02
BOLD=\\x01$(tput bold)\\x02
NORMAL=\\x01$(tput sgr0)\\x02
NORMAL_B=\\[$(tput sgr0)\\]

# Generates a pretty bash prompt.
generate_prompt() {

	# Get username and hostname.
	PROMPT_USER=$(whoami)
	PROMPT_HOST=$(hostname -s)

	# Get current working directory,
	# abbreviating HOME to ~ and replacing
	# path names with the first character of
	# their path (ignoring . in dotfiles).
	PROMPT_PATH=$(sed "s:\([^/\.]\)[^/]*/:\1/:g" <<< ${PWD/#$HOME/\~})

	# Get Git info, if any.
	PROMPT_GIT=$(__git_ps1 "${NORMAL}${FAINT}(git: ${MAGENTA}%s${FAINT})")

	# Print prompt.
	echo -e "${GREEN}${PROMPT_USER}${NORMAL}${FAINT}@${GREEN}${PROMPT_HOST} ${BOLD}${PURPLE}${PROMPT_PATH} ${PROMPT_GIT}\n${BOLD}${PURPLE}>${NORMAL}${PURPLE} "
}

# Install bash prompt customizations.
PS1='$(generate_prompt)'
PS2="${NORMAL_B}${MAGENTA_B}> "
PS0="${NORMAL_B}"

# Insert a newline after every command executed,
# except the clear command (since we don't want
# newlines at the top of an empty terminal).
PROMPT_COMMAND="export PROMPT_COMMAND=echo"
alias clear="unset PROMPT_COMMAND; clear; PROMPT_COMMAND='export PROMPT_COMMAND=echo'"