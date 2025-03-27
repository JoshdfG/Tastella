# Variables (override these when running make)
WALLET ?= xion1quzuep30mjz9vfu6mevgy4tcempf3sh9ag3zwk
CONTRACT ?= xion1sljvzt7clk2l5aytf8ur5sllafaphd8v2ed435kv85mjdncts4tqc8hfr7
NODE ?= https://rpc.xion-testnet-2.burnt.com:443
CHAIN_ID ?= xion-testnet-2
MIGRATE_MSG ?= {"update_riders_and_users":{}}

.PHONY: all optimize store migrate verify clean help

all: migrate

optimize:
	@echo "\033[34mOptimizing tastella.wasm...\033[0m"
	docker run --rm -v "$(PWD)":/code \
		--mount type=volume,source="$(shell basename "$(PWD)")_cache",target=/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		cosmwasm/optimizer:0.16.0

store: optimize
	@echo "\033[34mStoring tastella.wasm on Xion testnet...\033[0m"
	@RES=$$(xiond tx wasm store ./artifacts/tastella.wasm \
		--chain-id "$(CHAIN_ID)" \
		--gas-adjustment 1.3 \
		--gas-prices 0.1uxion \
		--gas auto \
		-y --output json \
		--node "$(NODE)" \
		--from "$(WALLET)"); \
	TXHASH=$$(echo "$$RES" | jq -r '.txhash'); \
	if [ -z "$$TXHASH" ] || [ "$$TXHASH" = "null" ]; then \
		echo "Error: Failed to get txhash from wasm store"; \
		exit 1; \
	fi; \
	echo "Wasm store txhash: \033[32m$$TXHASH\033[0m"; \
	echo "$$TXHASH" > .upgrade_txhash.tmp; \
	sleep 10; \
	NEW_CODE_ID=$$(xiond query tx "$$TXHASH" --node "$(NODE)" -o json | jq -r '.events[] | select(.type == "store_code") | .attributes[] | select(.key == "code_id") | .value'); \
	if [ -z "$$NEW_CODE_ID" ] || [ "$$NEW_CODE_ID" = "null" ]; then \
		echo "Error: Failed to get NEW_CODE_ID from tx $$TXHASH"; \
		exit 1; \
	fi; \
	echo "New CODE_ID: \033[32m$$NEW_CODE_ID\033[0m"; \
	echo "$$NEW_CODE_ID" > .upgrade_code_id.tmp

migrate: store
	@echo "\033[34mMigrating contract $(CONTRACT) to code ID $$(cat .upgrade_code_id.tmp)...\033[0m"
	@MIGRATE_RES=$$(xiond tx wasm migrate "$(CONTRACT)" "$$(cat .upgrade_code_id.tmp)" '$(MIGRATE_MSG)' \
		--from "$(WALLET)" \
		--gas-prices 0.025uxion \
		--gas auto \
		--gas-adjustment 1.3 \
		-y \
		--chain-id "$(CHAIN_ID)" \
		--node "$(NODE)" --output json); \
	MIGRATE_TXHASH=$$(echo "$$MIGRATE_RES" | jq -r '.txhash'); \
	if [ -z "$$MIGRATE_TXHASH" ] || [ "$$MIGRATE_TXHASH" = "null" ]; then \
		echo "Error: Failed to get txhash from migrate"; \
		exit 1; \
	fi; \
	echo "Migrate txhash: \033[32m$$MIGRATE_TXHASH\033[0m"; \
	echo "$$MIGRATE_TXHASH" > .upgrade_migrate_txhash.tmp

verify: migrate
	@echo "\033[34mVerifying migration...\033[0m"
	@xiond query tx "$$(cat .upgrade_migrate_txhash.tmp)" --node "$(NODE)" -o json | jq .

clean:
	@rm -f .upgrade_txhash.tmp .upgrade_code_id.tmp .upgrade_migrate_txhash.tmp

help:
	@echo "Available targets:"
	@echo "  all      : Run migrate (default)"
	@echo "  optimize : Optimize the wasm file"
	@echo "  store    : Store the wasm on chain"
	@echo "  migrate  : Migrate the contract"
	@echo "  verify   : Verify the migration"
	@echo "  clean    : Remove temporary files"
	@echo "  help     : Show this help message"


	.PHONY: all schema schema-release clean help

# Default target
all: schema

# Generate schema with dev profile (normal)
schema:
	@echo "\033[34mGenerating schema with dev profile...\033[0m"
	cargo run --bin schema
	@echo "\033[32mSchema files generated (dev profile)\033[0m"

# Generate schema with release profile (optimized)
schema-release:
	@echo "\033[34mGenerating schema with release profile...\033[0m"
	cargo run --release --bin schema
	@echo "\033[32mSchema files generated (release profile)\033[0m"

# Clean up generated schema files
clean:
	@rm -f *.json
	@echo "\033[32mCleaned up schema JSON files\033[0m"

# Help
help:
	@echo "Available targets:"
	@echo "  all           : Run schema (default, dev profile)"
	@echo "  schema        : Generate schema with dev profile (normal)"
	@echo "  schema-release: Generate schema with release profile (optimized)"
	@echo "  clean         : Remove generated JSON files"
	@echo "  help          : Show this help message"