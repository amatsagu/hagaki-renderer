build:
	/home/ubuntu/.cargo/bin/cargo build --release

install-service:
	chown ubuntu:ubuntu target/release/hagaki
	cp "/home/ubuntu/hagaki/app.service" "/lib/systemd/system/hagaki.service"
	chmod 644 "/lib/systemd/system/hagaki.service"
	systemctl daemon-reload
	systemctl enable hagaki.service

remove-service:
	systemctl stop hagaki.service
	rm "/lib/systemd/system/hagaki.service"
	systemctl daemon-reload
	# journalctl --rotate
	# journalctl --vacuum-time=1s

reload-service:
	chown ubuntu:ubuntu target/release/hagaki
	systemctl restart hagaki.service
	systemctl status hagaki.service
