case "${1:-build}" in
  build|test) echo ok ;;
  *) echo unknown ;;
esac

outer() {
  local value
  value=$(printf '%s' "$1")
  for part in "$@"; do
    printf '%s\n' "$part"
  done
  inner() { echo "$value"; }
  inner
}

helper() {
  while read -r line; do
    echo "$line"
  done < <(printf '%s\n' a b)
}
