import sqlite3
c = sqlite3.connect("osv.db")
d = c.cursor()
d.executescript("""CREATE TABLE Topics (
	title TEXT NOT NULL,
	admin TEXT NOT NULL,
	topic_id TEXT NOT NULL
);


CREATE TABLE Posts (
	body TEXT NOT NULL,
	name TEXT NOT NULL,
	ip TEXT NOT NULL,
	timestamp TEXT NOT NULL,
    topic_id TEXT NOT NULL
);
""")
c.commit()
c.close()