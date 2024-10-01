filename=".miri_allowlist"
while IFS= read -r line; do
if [[ "$line" == \#* ]]; then
    continue
fi          
if [[ -d $line ]]; then
    cd "$line" || { echo "Failed to change directory to $line"; exit 1; }
    echo "Run cargo miri test under: $(pwd)"
    cargo miri test
    cd -
else
    echo "$line is not a valid directory."
fi
done < "$filename"     