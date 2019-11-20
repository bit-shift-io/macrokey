#ifndef EVENT_DEVICE_H
#define EVENT_DEVICE_H

/**
 * Structure for input devices
 */

#include <iostream>
#include <stdio.h>
#include <linux/uhid.h>
#include <fcntl.h>   // open
#include <unistd.h>  // daemon, close

class event_device {
public:
    std::string device;
    std::string name;
    //int id;

    bool lock;
    int fd; // filedevice

    event_device(std::string p_device) {
        fd = -1;
        //id = -1;
        device = p_device;

        // read name
        char nm[256] = "???";
        fd = ::open(device.c_str(), O_RDONLY);
        if (fd < 0)
            return;
        ioctl(fd, EVIOCGNAME(sizeof(nm)), nm);

        name = nm;
        fprintf(stderr, "Device: %s, %s\n", device.c_str(), name.c_str());

        ::close(fd);
    }

    void open(bool p_lock){
        fprintf(stderr, "Opening Device: %s, %s\n", device.c_str(), name.c_str());

        lock = p_lock;
        // O_RDWR, O_RDONLY
        fd = ::open(device.c_str(), O_RDONLY);
        if (fd == -1) {
            fprintf(stderr, "Failed to open event device: %s.\n", device.c_str());
            exit;
        }

        // lock/grab all input
        if (lock) {
            int result = 0;
            result = ioctl(fd, EVIOCGRAB, 1);
            fprintf(stderr, "Exclusive access: %s\n", (result == 0) ? "SUCCESS" : "FAILURE");
        }
    }

    void close() {
        int result = ioctl(fd, EVIOCGRAB, 0);
        ::close(fd);
    }

    bool is_open() { return fd != -1; }
};

#endif // EVENT_DEVICE_H