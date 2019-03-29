#!/bin/bash

set -e

echo "==== Downloading paperpi binary ===="
curl -L -o /usr/local/bin/paperpi 'https://github.com/mprasil/paperpi/releases/latest/download/paperpi'
chmod a+x /usr/local/bin/paperpi

echo "==== Creating launch script ===="
read -p "Please provide Paperspace token now: " API_TOKEN

cat > Paperspace.sh <<'EOF'
#!/bin/bash

readonly JOY2KEY_SCRIPTLOC="$HOME/RetroPie-Setup/scriptmodules/helpers.sh"


# Stop old instance if left running
sudo pkill -f joy2key
sleep 0.5

if [[ -f $JOY2KEY_SCRIPTLOC ]]; then
    source "$JOY2KEY_SCRIPTLOC"
    scriptdir="$HOME/RetroPie-Setup"
    joy2keyStart
else
    echo "Warning: Can't import Joy2Key Script! You might need to use keyboard instead."
    echo "Script not found in: $JOY2KEY_SCRIPTLOC"
fi

EOF

echo "paperpi $API_TOKEN" >> Paperspace.sh
echo "joy2keyStop" >> Paperspace.sh

chmod a+x Paperspace.sh

echo "==== Done ===="
echo "Don't forget to restart Emulation station now"
