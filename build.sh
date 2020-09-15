# Hacky build script cos my Theos is fucked
rm -r ./packages
make clean package
dpkg-deb -Z gzip -b .theos/_ ./package.deb
tput bel
scp ./package.deb root@192.168.1.226:/User/Downloads/package.deb
tput bel
ssh root@192.168.1.226 "dpkg -i /User/Downloads/package.deb && killall -9 gta3sa"
