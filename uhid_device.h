#ifndef UHID_UTIL_H
#define UHID_UTIL_H

class uhid_device{
private:
    int fd;

    int uhid_write(const struct uhid_event *ev);
    int create();
    void destroy();
    void handle_output(struct uhid_event *ev);
    int event();
    int keyboard();

public:
    uhid_device();
    int send_event(uhid_event *ev);
};

#endif // UHID_UTIL_H
