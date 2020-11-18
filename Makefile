.PHONEY=
.PHONEY=all install

BUILD_ROOT=build

all:
	cargo install --path . --root=${BUILD_ROOT}

install:
	install -D ${BUILD_ROOT}/bin/libvirtd-wol /usr/local/bin/
	setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/libvirtd-wol
	install -D libvirtd-wol.service /etc/systemd/system/
	systemctl daemon-reload
	systemctl restart libvirtd-wol.service
