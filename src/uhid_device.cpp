
/*
 * Links
 * https://www.kernel.org/doc/html/latest/usb/gadget_hid.html
 * https://github.com/aagallag/hid_gadget_test
 * https://elixir.bootlin.com/linux/latest/source/drivers/hid/usbhid
 */

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


#include <pthread.h>
#include <string.h>
#include <stdio.h>
#include <ctype.h>
#include <fcntl.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

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

#if 0
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
#endif

/*
// https://www.spinics.net/lists/linux-usb/msg165574.html
// USB HID report descriptor
// https://forums.obdev.at/viewtopic.php?t=10780
static unsigned char rdesc[] = {
	0x05, 0x01,
	0x09, 0x06,
	0xa1, 0x01,

	0x05, 0x07,
	0x19, 0xe0,
	0x29, 0xe7,
	0x15, 0x00,
	0x25, 0x01,
	0x75, 0x01,
	0x95, 0x08,
	0x81, 0x02,

	0x95, 0x01,
	0x75, 0x08,
	0x81, 0x03,

	0x95, 0x05,
	0x75, 0x01,
	0x05, 0x08,
	0x19, 0x01,
	0x29, 0x05,
	0x91, 0x02,

	0x95, 0x01,
	0x75, 0x03,
	0x91, 0x03,

	0x95, 0x06,
	0x75, 0x08,
	0x15, 0x00,
	0x25, 0x65,
	0x05, 0x07,
	0x19, 0x00,
	0x29, 0x65,
	0x81, 0x00,

	0xc0
};

*/
typedef struct{
   unsigned char   reportID;
    unsigned char   buttonMask;
    unsigned char   dx;
    unsigned char   dy;
    unsigned char   dWheel;
} mouse_report_t;

typedef struct{
   unsigned char   reportID;
   unsigned char   modifier;
   unsigned char   reserved;
   unsigned char   keycode[5];
} keyboard_report_t;

#define ID_KEYBOARD 2
#define ID_MOUSE 1

static unsigned char rdesc[] = {
   //45 47
   0x05, 0x01,                    // USAGE_PAGE (Generic Desktop)
   0x09, 0x06,                    // USAGE (Keyboard)
   0xa1, 0x01,                    // COLLECTION (Application)
   0x85, (unsigned char)ID_KEYBOARD,    //   REPORT_ID (2)
   0x05, 0x07,                    //   USAGE_PAGE (Keyboard)
   0x19, 0xe0,                    //   USAGE_MINIMUM (Keyboard LeftControl)
   0x29, 0xe7,                    //   USAGE_MAXIMUM (Keyboard Right GUI)
   0x15, 0x00,                    //   LOGICAL_MINIMUM (0)
   0x25, 0x01,                    //   LOGICAL_MAXIMUM (1)
   0x95, 0x08,                    //   REPORT_COUNT (8)
   0x75, 0x01,                    //   REPORT_SIZE (1)
   0x81, 0x02,                    //   INPUT (Data,Var,Abs)
   0x95, 0x01,                    //   REPORT_COUNT (1)
   0x75, 0x08,                    //   REPORT_SIZE (8)
   0x81, 0x01,                    //   INPUT (Cnst,Ary,Abs)
   0x95, 0x05,                    //   REPORT_COUNT (6)
   0x75, 0x08,                    //   REPORT_SIZE (8)
   0x15, 0x00,                    //   LOGICAL_MINIMUM (0)
   0x25, 0x65,                    //   LOGICAL_MAXIMUM (101)
   0x05, 0x07,                    //   USAGE_PAGE (Keyboard)
   0x19, 0x00,                    //   USAGE_MINIMUM (Reserved (no event indicated))
   0x29, 0x65,                    //   USAGE_MAXIMUM (Keyboard Application)
   0x81, 0x00,                    //   INPUT (Data,Ary,Abs)
   0xc0,                           // END_COLLECTION
   
   //52 54
   0x05, 0x01,                    // USAGE_PAGE (Generic Desktop)
   0x09, 0x02,                    // USAGE (Mouse)
   0xa1, 0x01,                    // COLLECTION (Application)
   0x85, (unsigned char)ID_MOUSE,       //   REPORT_ID (1)
   0x09, 0x01,                    //   USAGE (Pointer)
   0xA1, 0x00,                    //   COLLECTION (Physical)
   0x05, 0x09,                    //     USAGE_PAGE (Button)
   0x19, 0x01,                    //     USAGE_MINIMUM
   0x29, 0x03,                    //     USAGE_MAXIMUM
   0x15, 0x00,                    //     LOGICAL_MINIMUM (0)
   0x25, 0x01,                    //     LOGICAL_MAXIMUM (1)
   0x95, 0x03,                    //     REPORT_COUNT (3)
   0x75, 0x01,                    //     REPORT_SIZE (1)
   0x81, 0x02,                    //     INPUT (Data,Var,Abs)
   0x95, 0x01,                    //     REPORT_COUNT (1)
   0x75, 0x05,                    //     REPORT_SIZE (5)
   0x81, 0x03,                    //     INPUT (Const,Var,Abs)
   0x05, 0x01,                    //     USAGE_PAGE (Generic Desktop)
   0x09, 0x30,                    //     USAGE (X)
   0x09, 0x31,                    //     USAGE (Y)
   0x09, 0x38,                    //     USAGE (Wheel)
   0x15, 0x81,                    //     LOGICAL_MINIMUM (-127)
   0x25, 0x7F,                    //     LOGICAL_MAXIMUM (127)
   0x75, 0x08,                    //     REPORT_SIZE (8)
   0x95, 0x03,                    //     REPORT_COUNT (3)
   0x81, 0x06,                    //     INPUT (Data,Var,Rel)
   0xC0,                          //   END_COLLECTION
   0xC0,                     // END COLLECTION   
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

struct options {
	const char    *opt;
	unsigned char val;
};

static struct options mmod[] = {
	{.opt = "--b1", .val = 0x01},
	{.opt = "--b2", .val = 0x02},
	{.opt = "--b3", .val = 0x04},
	{.opt = NULL}
};

int mouse_fill_report(char report[8], char buf[BUF_LEN], int *hold)
{
	char *tok = strtok(buf, " ");
	int mvt = 0;
	int i = 0;
	for (; tok != NULL; tok = strtok(NULL, " ")) {

		if (strcmp(tok, "--quit") == 0)
			return -1;

		if (strcmp(tok, "--hold") == 0) {
			*hold = 1;
			continue;
		}

		for (i = 0; mmod[i].opt != NULL; i++)
			if (strcmp(tok, mmod[i].opt) == 0) {
				report[0] = report[0] | mmod[i].val;
				break;
			}
		if (mmod[i].opt != NULL)
			continue;

		if (!(tok[0] == '-' && tok[1] == '-') && mvt < 2) {
			errno = 0;
			report[1 + mvt++] = (char)strtol(tok, NULL, 0);
			if (errno != 0) {
				fprintf(stderr, "Bad value:'%s'\n", tok);
				report[1 + mvt--] = 0;
			}
			continue;
		}

		fprintf(stderr, "unknown option: %s\n", tok);
	}
	return 3;
}

static struct options kmod[] = {
	{.opt = "--left-ctrl",		.val = 0x01},
	{.opt = "--right-ctrl",		.val = 0x10},
	{.opt = "--left-shift",		.val = 0x02},
	{.opt = "--right-shift",	.val = 0x20},
	{.opt = "--left-alt",		.val = 0x04},
	{.opt = "--right-alt",		.val = 0x40},
	{.opt = "--left-meta",		.val = 0x08},
	{.opt = "--right-meta",		.val = 0x80},
	{.opt = NULL}
};

static struct options kval[] = {
	{.opt = "--return",	.val = 0x28},
	{.opt = "--esc",	.val = 0x29},
	{.opt = "--bckspc",	.val = 0x2a},
	{.opt = "--tab",	.val = 0x2b},
	{.opt = "--spacebar",	.val = 0x2c},
	{.opt = "--caps-lock",	.val = 0x39},
	{.opt = "--f1",		.val = 0x3a},
	{.opt = "--f2",		.val = 0x3b},
	{.opt = "--f3",		.val = 0x3c},
	{.opt = "--f4",		.val = 0x3d},
	{.opt = "--f5",		.val = 0x3e},
	{.opt = "--f6",		.val = 0x3f},
	{.opt = "--f7",		.val = 0x40},
	{.opt = "--f8",		.val = 0x41},
	{.opt = "--f9",		.val = 0x42},
	{.opt = "--f10",	.val = 0x43},
	{.opt = "--f11",	.val = 0x44},
	{.opt = "--f12",	.val = 0x45},
	{.opt = "--insert",	.val = 0x49},
	{.opt = "--home",	.val = 0x4a},
	{.opt = "--pageup",	.val = 0x4b},
	{.opt = "--del",	.val = 0x4c},
	{.opt = "--end",	.val = 0x4d},
	{.opt = "--pagedown",	.val = 0x4e},
	{.opt = "--right",	.val = 0x4f},
	{.opt = "--left",	.val = 0x50},
	{.opt = "--down",	.val = 0x51},
	{.opt = "--kp-enter",	.val = 0x58},
	{.opt = "--up",		.val = 0x52},
	{.opt = "--num-lock",	.val = 0x53},
	{.opt = NULL}
};

int keyboard_fill_report(char report[8], char buf[BUF_LEN], int *hold)
{
	char *tok = strtok(buf, " ");
	int key = 0;
	int i = 0;

	for (; tok != NULL; tok = strtok(NULL, " ")) {

		if (strcmp(tok, "--quit") == 0)
			return -1;

		if (strcmp(tok, "--hold") == 0) {
			*hold = 1;
			continue;
		}

		if (key < 6) {
			for (i = 0; kval[i].opt != NULL; i++)
				if (strcmp(tok, kval[i].opt) == 0) {
					report[2 + key++] = kval[i].val;
					break;
				}
			if (kval[i].opt != NULL)
				continue;
		}

		if (key < 6)
			if (islower(tok[0])) {
				report[2 + key++] = (tok[0] - ('a' - 0x04));
				continue;
			}

		for (i = 0; kmod[i].opt != NULL; i++)
			if (strcmp(tok, kmod[i].opt) == 0) {
				report[0] = report[0] | kmod[i].val;
				break;
			}
		if (kmod[i].opt != NULL)
			continue;

		if (key < 6)
			fprintf(stderr, "unknown option: %s\n", tok);
	}
	return 8;
}

/*
UHID_INPUT2:
Same as UHID_INPUT, but the data array is the last field of uhid_input2_req.
Enables userspace to write only the required bytes to kernel (ev.type +
ev.u.input2.size + the part of the data array that matters), instead of
the entire struct uhid_input2_req.
*/
int uhid_device::send_event(int p_key, int p_state) {
    printf("send_event: %i", p_key);

    if (p_key == BTN_LEFT || p_key == BTN_RIGHT || p_key == BTN_MIDDLE) {
        printf("trying to mouse emulate: %i\n", p_key);
        
        int buttonMask = 0;
        switch (p_key) {
        case BTN_LEFT:
            buttonMask = 0x1;
            break;

        case BTN_RIGHT:
            buttonMask = 0x2;
            break;

        case BTN_MIDDLE:
            buttonMask = 0x4;
            break;
        }

        mouse_report_t report;
        const int size = sizeof(report);
        memset(&report, 0x0, size);
        report.reportID = ID_MOUSE;

        if (p_state == EV_PRESSED)
            report.buttonMask |= buttonMask;
        else if (p_state == EV_RELEASED)
            report.buttonMask &= ~buttonMask;

        struct uhid_event ev;
        ev.type = UHID_INPUT2;
        ev.u.input2.size = size;
        memcpy(ev.u.input2.data, &report, size);
        return uhid_write(&ev);
    }
    else {
        printf("trying to keyboard emulate: %i\n", p_key);

        keyboard_report_t report;
        const int size = sizeof(report);
        memset(&report, 0x0, size);
        report.reportID = ID_KEYBOARD;

        if (p_state == EV_PRESSED)
            report.keycode[0] |= p_key;
        else if (p_state == EV_RELEASED)
            report.keycode[0] &= ~p_key;

        struct uhid_event ev;
        ev.type = UHID_INPUT2;
        ev.u.input2.size = size;
        memcpy(ev.u.input2.data, &report, size);
        return uhid_write(&ev);
    }

    return -1;

#if 0

    switch (p_key) {
    case BTN_LEFT:
    case BTN_RIGHT:
    case BTN_MIDDLE:
        {

            /*
            char report[8];
            memset(report, 0x0, sizeof(report));

            if (p_state == EV_PRESSED)
                report[0] |= 0x1;
            else if (p_state == EV_RELEASED)
                report[0] &= ~0x1;
            */
/*
            int hold = 0;
            char report[8];
            char buf[BUF_LEN] = "--b1";
            memset(report, 0x0, sizeof(report));
            int to_send = mouse_fill_report(report, buf, &hold);

            if (write(fd, report, to_send) != to_send) {
				perror(filename);
				return 5;
			}
*/
/*
            if (p_state == EV_PRESSED)
                state.u.input2.data[1] |= p_key;
            else if (p_state == EV_RELEASED)
                state.u.input2.data[1] &= ~p_key;
            break;
*/

/*
            int hold = 0;
            char report[8];
            char buf[BUF_LEN] = "--spacebar";
            memset(report, 0x0, sizeof(report));
            int to_send = keyboard_fill_report(report, buf, &hold);
*/

            if (p_state == EV_PRESSED)
                state.u.input2.data[1] |= 0x1;
            else if (p_state == EV_RELEASED)
                state.u.input2.data[1] &= ~0x1;


            mouse_report_t report;
            const int size = sizeof(report);
            memset(&report, 0x0, size);
            report.reportID = ID_MOUSE;

            if (p_state == EV_PRESSED)
                report.buttonMask |= 0x1;
            else if (p_state == EV_RELEASED)
                report.buttonMask &= ~0x1;

            struct uhid_event ev;
            ev.type = UHID_INPUT2;
            ev.u.input2.size = size;
            memcpy(ev.u.input2.data, &report, size);
            return uhid_write(&ev);

            break;
        }

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

    default: {
            printf("trying to emulate: %i\n", p_key);

            keyboard_report_t report;
            const int size = sizeof(report);
            memset(&report, 0x0, size);
            report.reportID = ID_KEYBOARD;

            if (p_state == EV_PRESSED)
                report.keycode[0] |= p_key;
            else if (p_state == EV_RELEASED)
                report.keycode[0] &= ~p_key;

            struct uhid_event ev;
            ev.type = UHID_INPUT2;
            ev.u.input2.size = size;
            memcpy(ev.u.input2.data, &report, size);
            return uhid_write(&ev);
        }
    }
    
    return send_event(&state);
#endif
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

