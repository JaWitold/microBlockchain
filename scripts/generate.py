import yaml
import sys

def generate_docker_compose(n: int, prefixes: dict = {
    "rust": "rusty_miner",
    "redis": "redis",
    "rabbitmq": "rabbitmq",
    "nextjs": "nextjs" }
):
    if not isinstance(n, int) or n <= 0 or n >= 240:
        print("Error: n must be a positive integer less than 240")
        return

    docker_compose = {
        "version": "3",
        "services": {},
        "networks": {
            "blockchain-network": {
                "name": "blockchain-p2p-network",
                "ipam": {
                    "driver": "default",
                    "config": [
                        {
                            "subnet": "172.20.0.0/16"
                        }
                    ]
                }
            },
            "default": {
                "name": "blockchain-default-network"
            }
        },
    }

    for i in range(1, n+1): 
        # Use prefixes["rust"]
        miner_node_name = f"{prefixes['rust']}_{i}"
        redis_node_name = f"{prefixes['redis']}_{i}"
        rabbitmq_node_name = f"{prefixes['rabbitmq']}_{i}"
        nextjs_node_name = f"{prefixes['nextjs']}_{i}"
        internal_network_name = f"blockchain-internal-app-network-{i}"

        # miner
        docker_compose["services"][miner_node_name] = {
            "build": {
                "context": "./backend",
                "dockerfile": "Dockerfile"
            },
            "container_name": miner_node_name,
            "environment": [
                "CARGO_TARGET_DIR=/app/target",
                "RUST_LOG=info",
                f"REDIS_DSN=redis://redis_{i}:6379",
                f"RABBITMQ_DSN=amqp://guest:guest@{rabbitmq_node_name}:5672",
                f"GROUP={i}",
            ],
            "expose": [
                "8080"
            ],
            "networks": {
                "blockchain-network": {
                    "ipv4_address": f"172.20.0.{i+1}"
                },
                internal_network_name: {}
            },
            # "depends_on": [
            #     rabbitmq_node_name,
            #     redis_node_name
            # ],
            "restart": "always"
        }

        # redis
        docker_compose["services"][redis_node_name] = {
            "container_name": redis_node_name,
            "image": "redis:7.0-alpine3.17",
            "networks": {
                internal_network_name: {}
            },
            "ports": [
                f"{6379+i}:6379",
            ],
            "restart": "always",
        }

        # RabbitMQ
        docker_compose["services"][rabbitmq_node_name] = {
            "container_name": rabbitmq_node_name,
            "image": "rabbitmq:3.11.15-management-alpine",
            "networks": {
                "default": {},
                internal_network_name: {},
            },
            "restart": "unless-stopped",
            "ports": [
                f"{5672+i}:5672",
                f"{15672+i}:15672"
            ],
            "environment": [
                "RABBITMQ_DEFAULT_USER=guest",
                "RABBITMQ_DEFAULT_PASS=guest",
            ],
            "volumes": [
                "./docker/rabbitmq:/etc/rabbitmq"
            ],
            # "command": "rabbitmq-server -c /etc/rabbitmq/rabbitmq.conf"
        }

        # nextjs
        docker_compose["services"][nextjs_node_name] = {
            "build": {
                "context": "./frontend",
                "dockerfile": "Dockerfile.dev"
            },
            "environment": [
                f"REDIS_URL=redis://{redis_node_name}:6379",
                f"RABBITMQ_URL=amqp://guest:guest@{rabbitmq_node_name}:5672",
                "DEFAULT_QUEUE_NAME=blockchain-data",
                f"GROUP={i}",
                "NODE_ENV=development",
            ],
            "volumes": [
                "./frontend:/app",
                "/app/node_modules"
            ],
            "container_name": nextjs_node_name,
            "ports": [
                f"{3000 + i}:3000"
            ],
            "networks": {
                "default": {},
                internal_network_name: {},
            },
        }

        # internal network
        docker_compose["networks"][internal_network_name] = {
            "name": internal_network_name,
            "internal": True,
        }
        
        # if i != 1:
        #     docker_compose["services"][miner_node_name]["depends_on"] = ["node1"]


    docker_compose["volumes"] = {
        "rust_crate_index": {}
    }

    with open("docker-compose.gen.yaml", "w") as f:
        yaml.dump(docker_compose, f)

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print('Error: n must be provided as command line argument')
    else:
        try:
            n = int(sys.argv[1])
            prefixes = {
                "rust": sys.argv[2],
                "redis": sys.argv[3],
                "rabbitmq": sys.argv[4],
                "nextjs": sys.argv[5]
            }
            generate_docker_compose(n, prefixes)
        except ValueError:
            print('Error: n must be an integer')
        except IndexError:
            print('No prefixes provided, using default prefixes:' \
                  'rusty_miner", "redis", "rabbitmq", "nextjs"')
            generate_docker_compose(n)
