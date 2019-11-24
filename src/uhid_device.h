#ifndef UHID_DEVICE_H
#define UHID_DEVICE_H

/**
 * Legacy code, left for reference
 * 
 * Links:
 * https://cgit.freedesktop.org/~whot/evtest/tree/evtest.c
 * https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/Documentation/hid/uhid.txt?id=refs/tags/v4.10-rc3
 * https://github.com/torvalds/linux/blob/master/samples/uhid/uhid-example.c
 **/

// 0 = released, 1 = pressed, 2 repeat
#define EV_PRESSED 1
#define EV_RELEASED 0
#define EV_REPEAT 2

#define LEFT_MOUSE 0

#define BUF_LEN 512

class uhid_device {
private:
    int fd; // file device
    uhid_event state;
    int uhid_write(const struct uhid_event *ev);
    int create();
    void destroy();

public:
    uhid_device();
    ~uhid_device();

    int send_event(uhid_event *ev);
    int send_event(int key, int state);
};

#endif // UHID_DEVICE_H
