 
#ifndef UINPUT_DEVICE_H
#define UINPUT_DEVICE_H

/**
 * Superceeds uhid_device code as finding examples with that sucked!
 * 
 * Open a virtual keyboard and write out key events using uinput API
 * 
 * https://www.kernel.org/doc/html/v4.12/input/uinput.html
 * 
 */
class uinput_device {
protected:
    int fd;

public:
    uinput_device();
    ~uinput_device();

    void open();

    // state: 0 = released, 1 = pressed, 2 repeat
    int send_event(int key, int state);
};

#endif // UINPUT_DEVICE_H