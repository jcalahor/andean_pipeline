FROM python:3.11

WORKDIR /usr/src/andean_producer

COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

COPY . .
