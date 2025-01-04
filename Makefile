build:
	@echo "Compiling application..."
	/usr/local/go/bin/go build -o bin/maestro ./main.go
	@echo "Successfully compiled."

clean:
	@echo "Cleaning old service configuration..."
	systemctl stop maestro.service
	rm /home/ubuntu/maestro/bin/maestro
	@echo "Successfully cleaned old configuration."

install:
	@echo "Installing..."
	cp "/home/ubuntu/maestro/app.service" "/lib/systemd/system/maestro.service"
	chmod 644 "/lib/systemd/system/maestro.service"
	systemctl daemon-reload
	systemctl enable maestro.service
	@echo "Successfully installed."

reset:
	@echo "Attempting to re-install application..."
	$(MAKE) clean
	rm "/lib/systemd/system/maestro.service"
	systemctl daemon-reload
	journalctl --rotate
	journalctl --vacuum-time=1s
	chmod 777 "/home/ubuntu/maestro" -R
	$(MAKE) build
	systemctl restart maestro.service
	systemctl status maestro.service
	@echo "Successfully re-installed."

release:
	@echo "Running release script..."
	$(MAKE) clean
	$(MAKE) build
	systemctl restart maestro.service
	systemctl status maestro.service
	@echo "Successfully released."