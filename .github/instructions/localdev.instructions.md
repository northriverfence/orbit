Prereqs

Node/X, Python/Y, Docker/Z (pin exact versions here)

Copy env:

cp .env.example .env   # fill secrets locally


First run:

make bootstrap

Common tasks
make up          # start services
make test        # run unit tests
make lint        # lint / format check
make fix         # format and auto-fix
make down        # stop services

Troubleshooting

Port in use → make down && docker prune -f

DB migrations failed → make db.reset && make up
