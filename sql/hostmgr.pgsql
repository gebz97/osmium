DROP SCHEMA IF EXISTS hostmgr CASCADE;
CREATE SCHEMA hostmgr;

CREATE TABLE hostmgr.hosts (
    id SERIAL PRIMARY KEY,
    hostname VARCHAR(255) NOT NULL UNIQUE,
    fqdn VARCHAR(255),
    physical BOOLEAN,
    status VARCHAR(50) DEFAULT 'active',
    os_name VARCHAR(100),
    os_version VARCHAR(100),
    os_major VARCHAR(100),
    kernel_version VARCHAR(100),
    ip_address INET,
    subnet CIDR,
    environment VARCHAR(50),
    -- business_service VARCHAR(50) REFERENCES hostmgr.business_service(name),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE hostmgr.packages (
    id SERIAL PRIMARY KEY,
    host_id INTEGER NOT NULL REFERENCES hostmgr.hosts(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    version VARCHAR(100),
    UNIQUE(host_id, name)
);

CREATE TABLE hostmgr.users (
    id SERIAL PRIMARY KEY,
    host_id INTEGER NOT NULL REFERENCES hostmgr.hosts(id) ON DELETE CASCADE,
    username VARCHAR(100) NOT NULL,
    uid INTEGER,
    gid INTEGER,
    home_dir VARCHAR(255),
    shell VARCHAR(255),
    UNIQUE(host_id, username)
);

CREATE TABLE hostmgr.groups (
    id SERIAL PRIMARY KEY,
    host_id INTEGER NOT NULL REFERENCES hostmgr.hosts(id) ON DELETE CASCADE,
    groupname VARCHAR(100) NOT NULL,
    gid INTEGER,
    UNIQUE(host_id, groupname)
);

CREATE TABLE hostmgr.services (
    id SERIAL PRIMARY KEY,
    host_id INTEGER NOT NULL REFERENCES hostmgr.hosts(id) ON DELETE CASCADE,
    service_name VARCHAR(255) NOT NULL,
    status VARCHAR(50),
    enabled BOOLEAN,
    UNIQUE(host_id, service_name)
);

CREATE TABLE hostmgr.mounts (
    id SERIAL PRIMARY KEY,
    host_id INTEGER NOT NULL REFERENCES hostmgr.hosts(id) ON DELETE CASCADE,
    mount_point VARCHAR(255) NOT NULL,
    device VARCHAR(255),
    filesystem_type VARCHAR(50),
    total_size_gb BIGINT,
    used_gb BIGINT,
    UNIQUE(host_id, mount_point)
);

CREATE TABLE hostmgr.disks (
    id SERIAL PRIMARY KEY,
    host_id INTEGER NOT NULL REFERENCES hostmgr.hosts(id) ON DELETE CASCADE,
    device_name VARCHAR(50) NOT NULL,
    size_gb BIGINT,
    disk_type VARCHAR(50),
    UNIQUE(host_id, device_name)
);

CREATE TABLE hostmgr.netlinks (
    id SERIAL PRIMARY KEY,
    host_id INTEGER NOT NULL REFERENCES hostmgr.hosts(id) ON DELETE CASCADE,
    interface_name VARCHAR(50) NOT NULL,
    ip_address INET,
    mac_address MACADDR,
    status VARCHAR(50),
    UNIQUE(host_id, interface_name)
);

CREATE INDEX idx_hosts_hostname ON hostmgr.hosts(hostname);
CREATE INDEX idx_hosts_status ON hostmgr.hosts(status);
CREATE INDEX idx_packages_host_id ON hostmgr.packages(host_id);
CREATE INDEX idx_users_host_id ON hostmgr.users(host_id);
CREATE INDEX idx_groups_host_id ON hostmgr.groups(host_id);
CREATE INDEX idx_services_host_id ON hostmgr.services(host_id);
CREATE INDEX idx_mounts_host_id ON hostmgr.mounts(host_id);
CREATE INDEX idx_disks_host_id ON hostmgr.disks(host_id);
CREATE INDEX idx_netlinks_host_id ON hostmgr.netlinks(host_id);
