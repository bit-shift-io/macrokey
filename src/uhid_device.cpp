
/*
 * UHID Example
 * This example emulates a basic 3 buttons mouse with wheel over UHID. Run this
 * program as root and then use the following keys to control the mouse:
 *   q: Quit the application
 *   1: Toggle left button (down, up, ...)
 *   2: Toggle right button
 *   3: Toggle middle button
 *   a: Move mouse left
 *   d: Move mouse right
 *   w: Move mouse up
 *   s: Move mouse down
 *   r: Move wheel up
 *   f: Move wheel down
 *
 * Additionally to 3 button mouse, 3 keyboard LEDs are also supported (LED_NUML,
 * LED_CAPSL and LED_SCROLLL). The device doesn't generate any related keyboard
 * events, though. You need to manually write the EV_LED/LED_XY/1 activation
 * input event to the evdev device to see it being sent to this device.
 *
 * If uhid is not available as /dev/uhid, then you can pass a different path as
 * first argument.
 * If <linux/uhid.h> is not installed in /usr, then compile this with:
 *   gcc -o ./uhid_test -Wall -I./include ./samples/uhid/uhid-example.c
 * And ignore the warning about kernel headers. However, it is recommended to
 * use the installed uhid.h if available.
 */

#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <termios.h>
#include <unistd.h>
#include <linux/uhid.h>
#include <linux/input.h>

#include <src/uhid_device.h>

/*
 * HID Report Desciptor
 * We emulate a basic 3 button mouse with wheel and 3 keyboard LEDs. This is
 * the report-descriptor as the kernel will parse it:
 *
 * INPUT(1)[INPUT]
 *   Field(0)
 *     Physical(GenericDesktop.Pointer)
 *     Application(GenericDesktop.Mouse)
 *     Usage(3)
 *       Button.0001
 *       Button.0002
 *       Button.0003
 *     Logical Minimum(0)
 *     Logical Maximum(1)
 *     Report Size(1)
 *     Report Count(3)
 *     Report Offset(0)
 *     Flags( Variable Absolute )
 *   Field(1)
 *     Physical(GenericDesktop.Pointer)
 *     Application(GenericDesktop.Mouse)
 *     Usage(3)
 *       GenericDesktop.X
 *       GenericDesktop.Y
 *       GenericDesktop.Wheel
 *     Logical Minimum(-128)
 *     Logical Maximum(127)
 *     Report Size(8)
 *     Report Count(3)
 *     Report Offset(8)
 *     Flags( Variable Relative )
 * OUTPUT(2)[OUTPUT]
 *   Field(0)
 *     Application(GenericDesktop.Keyboard)
 *     Usage(3)
 *       LED.NumLock
 *       LED.CapsLock
 *       LED.ScrollLock
 *     Logical Minimum(0)
 *     Logical Maximum(1)
 *     Report Size(1)
 *     Report Count(3)
 *     Report Offset(0)
 *     Flags( Variable Absolute )
 *
 * This is the mapping that we expect:
 *   Button.0001 ---> Key.LeftBtn
 *   Button.0002 ---> Key.RightBtn
 *   Button.0003 ---> Key.MiddleBtn
 *   GenericDesktop.X ---> Relative.X
 *   GenericDesktop.Y ---> Relative.Y
 *   GenericDesktop.Wheel ---> Relative.Wheel
 *   LED.NumLock ---> LED.NumLock
 *   LED.CapsLock ---> LED.CapsLock
 *   LED.ScrollLock ---> LED.ScrollLock
 *
 * This information can be verified by reading /sys/kernel/debug/hid/<dev>/rdesc
 * This file should print the same information as showed above.
 */

// report descriptor data
static u_int8_t rdesc[] = {
    0x05, 0x01,	/* USAGE_PAGE (Generic Desktop) */
    0x09, 0x02,	/* USAGE (Mouse) */
    0xa1, 0x01,	/* COLLECTION (Application) */
    0x09, 0x01,		/* USAGE (Pointer) */
    0xa1, 0x00,		/* COLLECTION (Physical) */
    0x85, 0x01,			/* REPORT_ID (1) */
    0x05, 0x09,			/* USAGE_PAGE (Button) */
    0x19, 0x01,			/* USAGE_MINIMUM (Button 1) */
    0x29, 0x03,			/* USAGE_MAXIMUM (Button 3) */
    0x15, 0x00,			/* LOGICAL_MINIMUM (0) */
    0x25, 0x01,			/* LOGICAL_MAXIMUM (1) */
    0x95, 0x03,			/* REPORT_COUNT (3) */
    0x75, 0x01,			/* REPORT_SIZE (1) */
    0x81, 0x02,			/* INPUT (Data,Var,Abs) */
    0x95, 0x01,			/* REPORT_COUNT (1) */
    0x75, 0x05,			/* REPORT_SIZE (5) */
    0x81, 0x01,			/* INPUT (Cnst,Var,Abs) */
    0x05, 0x01,			/* USAGE_PAGE (Generic Desktop) */
    0x09, 0x30,			/* USAGE (X) */
    0x09, 0x31,			/* USAGE (Y) */
    0x09, 0x38,			/* USAGE (WHEEL) */
    0x15, 0x81,			/* LOGICAL_MINIMUM (-127) */
    0x25, 0x7f,			/* LOGICAL_MAXIMUM (127) */
    0x75, 0x08,			/* REPORT_SIZE (8) */
    0x95, 0x03,			/* REPORT_COUNT (3) */
    0x81, 0x06,			/* INPUT (Data,Var,Rel) */
    0xc0,			/* END_COLLECTION */
    0xc0,		/* END_COLLECTION */
    0x05, 0x01,	/* USAGE_PAGE (Generic Desktop) */
    0x09, 0x06,	/* USAGE (Keyboard) */
    0xa1, 0x01,	/* COLLECTION (Application) */
    0x85, 0x02,		/* REPORT_ID (2) */
    0x05, 0x08,		/* USAGE_PAGE (Led) */
    0x19, 0x01,		/* USAGE_MINIMUM (1) */
    0x29, 0x03,		/* USAGE_MAXIMUM (3) */
    0x15, 0x00,		/* LOGICAL_MINIMUM (0) */
    0x25, 0x01,		/* LOGICAL_MAXIMUM (1) */
    0x95, 0x03,		/* REPORT_COUNT (3) */
    0x75, 0x01,		/* REPORT_SIZE (1) */
    0x91, 0x02,		/* Output (Data,Var,Abs) */
    0x95, 0x01,		/* REPORT_COUNT (1) */
    0x75, 0x05,		/* REPORT_SIZE (5) */
    0x91, 0x01,		/* Output (Cnst,Var,Abs) */
    0xc0,		/* END_COLLECTION */
};

int uhid_device::uhid_write(const struct uhid_event *ev)
{
    ssize_t ret;

    ret = write(fd, ev, sizeof(*ev));
    if (ret < 0) {
        fprintf(stderr, "Cannot write to uhid: %m\n");
        return -errno;
    } else if (ret != sizeof(*ev)) {
        fprintf(stderr, "Wrong size written to uhid: %ld != %lu\n",
            ret, sizeof(ev));
        return -EFAULT;
    } else {
        return 0;
    }
}

int uhid_device::create()
{
    struct uhid_event ev;
    memset(&ev, 0, sizeof(ev));
    ev.type = UHID_CREATE2;
    strcpy((char*)ev.u.create2.name, "macrokey virtual uhid");
    memcpy(ev.u.create2.rd_data, rdesc, sizeof(rdesc));
    ev.u.create2.rd_size = sizeof(rdesc);
    ev.u.create2.bus = BUS_USB;
    ev.u.create2.vendor = 0x15d9;
    ev.u.create2.product = 0x0a37;
    ev.u.create2.version = 0;
    ev.u.create2.country = 0;

    return uhid_write(&ev);
}

/*
 * Destory the device
 */
void uhid_device::destroy()
{
    struct uhid_event ev;

    memset(&ev, 0, sizeof(ev));
    ev.type = UHID_DESTROY;

    uhid_write(&ev);
}

/*
 * Write event to device
 */
int uhid_device::send_event(uhid_event *ev)
{
    return uhid_write(ev);
}

/*
UHID_INPUT2:
Same as UHID_INPUT, but the data array is the last field of uhid_input2_req.
Enables userspace to write only the required bytes to kernel (ev.type +
ev.u.input2.size + the part of the data array that matters), instead of
the entire struct uhid_input2_req.
*/
int uhid_device::send_event(int p_key, int p_state) {

    switch (p_key) {
    case BTN_LEFT:
        if (p_state == EV_PRESSED)
            state.u.input2.data[1] |= 0x1;
        else if (p_state == EV_RELEASED)
            state.u.input2.data[1] &= ~0x1;
        break;

    case BTN_RIGHT:
        if (p_state == EV_PRESSED)
            state.u.input2.data[1] |= 0x2;
        else if (p_state == EV_RELEASED)
            state.u.input2.data[1] &= ~0x2;
        break;

    case BTN_MIDDLE:
        if (p_state == EV_PRESSED)
            state.u.input2.data[1] |= 0x4;
        else if (p_state == EV_RELEASED)
            state.u.input2.data[1] &= ~0x4;
        break;
    }

    return send_event(&state);
}

uhid_device::uhid_device()
{
    //int fd;
    const char *path = "/dev/uhid";
    //struct pollfd pfds[2];
    int ret;

    fprintf(stderr, "Virtual Device: %s\n", path);
    fd = open(path, O_RDWR | O_CLOEXEC);
    if (fd < 0) {
        fprintf(stderr, "Virtual Device: failed, %s: %m\n", path);
        //return EXIT_FAILURE;
    }

    // create device
    fprintf(stderr, "Virtual Device: create\n");
    ret = create();
    if (ret) {
        close(fd);
        //return EXIT_FAILURE;
    }


    // setup state
    memset(&state, 0, sizeof(state));
    state.type = UHID_INPUT2;
    state.u.input2.size = 5;
    state.u.input2.data[0] = 0x1;
}

uhid_device::~uhid_device()
{
    destroy();
}

