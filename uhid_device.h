#ifndef UHID_UTIL_H
#define UHID_UTIL_H

// 0 = released, 1 = pressed, 2 repeat
#define EV_PRESSED 1
#define EV_RELEASED 0
#define EV_REPEAT 2

#define LEFT_MOUSE 0

class uhid_device{
private:
    int fd;

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

#endif // UHID_UTIL_H
